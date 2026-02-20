use ../pw.nu

use ./common.nu *
use ./project.nu *

# Show active workspace/profile isolation bindings.
export def "pp isolate" []: nothing -> record {
    let workspace = ((pwd | path expand) | into string)
    let profile = (active-profile)
    let parsed = (pw session status --profile $profile)

    {
        workspace: $workspace
        profile: $profile
        active: ($parsed.data.active? | default false)
        session_key: ($parsed.data.session_key? | default null)
        workspace_id: ($parsed.data.workspace_id? | default null)
    }
}

# Set model mode via dropdown (Auto, Instant, Thinking, Pro)
# Uses single eval with polling since dropdowns close between pw commands
export def "pp set-model" [
    mode: string  # "auto", "instant", "thinking", or "pro"
]: nothing -> record {
    ensure-project-tab --navigate | ignore
    let mode_lower = ($mode | str downcase)
    let search_text = match $mode_lower {
        "auto" => "Decides how long"
        "instant" => "Answers right away"
        "thinking" => "Thinks longer"
        "pro" => "Research-grade intelligence"
        _ => { error make { msg: $"Unknown mode: ($mode). Use auto, instant, thinking, or pro." } }
    }

    let items_js = "(() => {
        const menu = document.querySelector('[role=\"menu\"]');
        if (!menu) return [];
        const normalize = (s) => (s || '').split('\\n').map(x => x.trim()).filter(Boolean).join(' ');
        return Array.from(menu.querySelectorAll('[role=\"menuitem\"]'))
            .filter(item => {
                const rect = item.getBoundingClientRect();
                return rect.width > 2 && rect.height > 2;
            })
            .map(item => ({
                text: normalize(item.textContent || ''),
                testid: item.getAttribute('data-testid') || null
            }));
    })()"

    mut items = []
    mut open_attempts = 0
    for _ in 1..4 {
        $open_attempts = $open_attempts + 1
        pw click "[data-testid=\"model-switcher-dropdown-button\"]" | ignore

        for _ in 1..20 {
            $items = ((pw eval $items_js).data.result | default [])
            if (($items | length) > 0) { break }
            sleep 50ms
        }

        if (($items | length) > 0) { break }
        sleep 100ms
    }
    if (($items | length) == 0) {
        error make { msg: $"Model menu did not open after ($open_attempts) attempts" }
    }

    let by_testid = match $mode_lower {
        "auto" => {
            $items
            | where { |item|
                let tid = ($item.testid | default "")
                ($tid | str starts-with "model-switcher-")
                and not ($tid | str ends-with "-instant")
                and not ($tid | str ends-with "-thinking")
                and not ($tid | str ends-with "-pro")
            }
            | first
        }
        "instant" => { $items | where { |item| (($item.testid | default "") | str ends-with "-instant") } | first }
        "thinking" => { $items | where { |item| (($item.testid | default "") | str ends-with "-thinking") } | first }
        "pro" => { $items | where { |item| (($item.testid | default "") | str ends-with "-pro") } | first }
        _ => null
    }

    let target = if ($by_testid | is-not-empty) {
        $by_testid
    } else {
        $items | where { |item| (($item.text | default "") | str contains $search_text) } | first
    }

    if ($target | is-empty) {
        error make {
            msg: $"Mode option not found in menu for mode '($mode_lower)'. Options: ($items | to json)"
        }
    }

    let target_testid = ($target.testid | default "")
    if ($target_testid | is-not-empty) {
        pw click $"[data-testid=\"($target_testid)\"]" | ignore
    } else {
        let search_text_json = ($search_text | to json)
        let click_by_text_js = "(() => {
            const menu = document.querySelector('[role=\"menu\"]');
            if (!menu) return { error: 'model menu not open' };
            const rows = Array.from(menu.querySelectorAll('[role=\"menuitem\"]'))
                .filter(item => {
                    const rect = item.getBoundingClientRect();
                    return rect.width > 2 && rect.height > 2;
                });
            const target = rows.find(item => (item.textContent || '').includes(" + $search_text_json + "));
            if (!target) return { error: 'mode option not found by text' };
            target.click();
            return { ok: true };
        })()"
        let click_result = (pw eval $click_by_text_js).data.result
        if ($click_result | get -o error | is-not-empty) {
            error make { msg: ($click_result.error) }
        }
    }

    sleep 300ms
    {
        success: true
        mode: $mode_lower
        selected_testid: (if ($target_testid | is-empty) { null } else { $target_testid })
        current: (get-current-model)
    }
}

# Refresh page (use when Navigator UI gets stuck)
export def "pp refresh" []: nothing -> record {
    ensure-project-tab --navigate | ignore
    pw eval "location.reload()"
    sleep 3sec
    { refreshed: true }
}

# Start a new temporary chat with the Navigator
export def "pp new" [
    --model (-m): string  # Model to set (auto, instant, thinking, pro). Defaults to thinking.
]: nothing -> record {
    let project = (configured-project)
    ensure-tab
    if ($project | is-not-empty) {
        pw nav $project.project_url | ignore
    } else {
        # Legacy fallback when no project is configured.
        pw nav $BASE_URL | ignore
        sleep 500ms
        pw nav $BASE_URL | ignore
    }
    pw wait-for "#prompt-textarea"
    sleep 500ms
    let mode = if ($model | is-empty) { $DEFAULT_MODEL } else { $model }
    pp set-model $mode
    { new_chat: true, model: (get-current-model) }
}

# Test helper: insert text into a selector without ensure-tab
export def "pp debug-insert" [
    text: string
    --selector (-s): string = "#prompt-textarea"
    --clear (-c)
]: nothing -> record {
    if $clear {
        insert-text $text --selector $selector --clear
    } else {
        insert-text $text --selector $selector
    }
}

# Paste text from stdin into Navigator composer (inline, no attachment)
export def "pp paste" [
    --send (-s)  # Also send after pasting
    --clear (-c) # Clear existing content first
]: string -> record {
    ensure-project-tab --navigate | ignore
    let text = $in

    if $send {
        let send_gate = (block-send-if-capped "pp paste --send")
        if (($send_gate.allowed? | default true) == false) {
            return {
                pasted: false
                sent: false
                blocked: true
                must_start_new: true
                reason: "conversation_cap_reached"
                length: 0
            }
        }
    }

    let result = if $clear { insert-text $text --clear } else { insert-text $text }

    if ($result | get -o error | is-not-empty) {
        error make { msg: ($result.error) }
    }

    if $send {
        pw eval "document.querySelector('[data-testid=\"send-button\"]')?.click()"
        maybe-warn-conversation-length "pp paste --send" | ignore
        { pasted: true, sent: true, length: ($result.inserted? | default ($result.length? | default 0)) }
    } else {
        { pasted: true, sent: false, length: ($result.inserted? | default ($result.length? | default 0)) }
    }
}
