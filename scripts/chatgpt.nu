# chatgpt.nu - ChatGPT automation workflows
#
# Usage:
#   use pw.nu
#   use chatgpt.nu *
#   chatgpt send "Explain quantum computing"
#   chatgpt set-model thinking

use pw.nu

const BASE_URL = "https://chatgpt.com"

# Get current model from selector aria-label
def get-current-model []: nothing -> string {
    let js = "(() => {
        const btn = document.querySelector(\"button[aria-label^='Model selector']\");
        if (!btn) return null;
        const match = btn.ariaLabel.match(/current model is (.+)/i);
        return match ? match[1] : null;
    })()"
    (pw eval $js).data.result
}

# Set model mode via dropdown (Auto, Instant, Thinking)
# Uses single eval with polling since dropdowns close between pw commands
export def "chatgpt set-model" [
    mode: string  # "auto", "instant", or "thinking"
]: nothing -> record {
    let mode_lower = ($mode | str downcase)
    let search_text = match $mode_lower {
        "auto" => "Decides how long"
        "instant" => "Answers right away"
        "thinking" => "Thinks longer"
        _ => { error make { msg: $"Unknown mode: ($mode). Use auto, instant, or thinking." } }
    }

    let js = "(async function() {
        const btn = document.querySelector(\"button[aria-label^='Model selector']\");
        if (!btn) return { error: \"Model selector not found\" };
        btn.click();

        // Async poll - yields to event loop so React can render
        for (let i = 0; i < 50; i++) {
            await new Promise(r => setTimeout(r, 10));
            const menu = document.querySelector(\"[role='menu']\");
            if (menu) {
                var items = menu.querySelectorAll(\"*\");
                for (var item of items) {
                    if (item.textContent.includes(\"" + $search_text + "\")) {
                        item.click();
                        return { success: true, mode: \"" + $mode_lower + "\" };
                    }
                }
                return { error: \"Mode option not found in menu\" };
            }
        }
        return { error: \"Menu did not open\" };
    })()"

    let result = (pw eval $js).data.result
    if ($result | get -o error | is-not-empty) {
        error make { msg: ($result.error) }
    }
    sleep 300ms
    { success: true, mode: $mode_lower, current: (get-current-model) }
}

# Check if extended thinking pill is visible
def has-thinking-pill []: nothing -> bool {
    let js = "(() => {
        const pill = document.querySelector(\".__composer-pill\");
        return pill !== null;
    })()"
    (pw eval $js).data.result
}

# Refresh page (use when ChatGPT UI gets stuck)
export def "chatgpt refresh" []: nothing -> record {
    pw eval "location.reload()"
    sleep 3sec
    { refreshed: true }
}

# Insert text into composer (bypasses attachment conversion for large text)
# Use execCommand which handles newlines and doesn't trigger file attachment
def insert-text [text: string]: nothing -> record {
    # Write text to temp file to avoid shell escaping issues
    let tmp = (mktemp)
    $text | save -f $tmp

    # Read and insert via JS
    let js_text = (open $tmp | to json)
    rm $tmp

    let js = "(function() {
        const el = document.querySelector('#prompt-textarea');
        if (!el) return { error: 'textarea not found' };
        el.focus();
        el.innerHTML = '';
        document.execCommand('insertText', false, " + $js_text + ");
        el.dispatchEvent(new InputEvent('input', { bubbles: true }));
        return { inserted: el.textContent.length };
    })()"

    (pw eval $js).data.result
}

# Paste text from stdin into ChatGPT composer (inline, no attachment)
export def "chatgpt paste" [
    --send (-s)  # Also send after pasting
]: string -> record {
    let text = $in
    let result = (insert-text $text)

    if ($result | get -o error | is-not-empty) {
        error make { msg: ($result.error) }
    }

    if $send {
        # Click send button
        pw eval "document.querySelector('[data-testid=\"send-button\"]')?.click()"
        { pasted: true, sent: true, length: $result.inserted }
    } else {
        { pasted: true, sent: false, length: $result.inserted }
    }
}

# Attach text as a document file (triggers ChatGPT's file attachment UI)
export def "chatgpt attach" [
    --name (-n): string = "document.txt"  # Filename for attachment
    --send (-s)  # Also send after attaching
]: string -> record {
    let text = $in

    # Write to temp file for JS to read
    let tmp = (mktemp)
    $text | save -f $tmp
    let js_text = (open $tmp | to json)
    let js_name = ($name | to json)
    rm $tmp

    let js = "(function() {
        const el = document.querySelector('#prompt-textarea');
        if (!el) return { error: 'textarea not found' };
        el.focus();

        const text = " + $js_text + ";
        const filename = " + $js_name + ";

        // Create file and DataTransfer
        const dt = new DataTransfer();
        const file = new File([text], filename, { type: 'text/plain' });
        dt.items.add(file);

        // Dispatch paste event with file
        const pasteEvent = new ClipboardEvent('paste', {
            bubbles: true,
            cancelable: true,
            clipboardData: dt
        });

        el.dispatchEvent(pasteEvent);
        return { attached: true, filename: filename, size: text.length };
    })()"

    let result = (pw eval $js).data.result

    if ($result | get -o error | is-not-empty) {
        error make { msg: ($result.error) }
    }

    # Wait for attachment to process
    sleep 500ms

    if $send {
        pw eval "document.querySelector('[data-testid=\"send-button\"]')?.click()"
        { attached: true, sent: true, filename: $name, size: ($text | str length) }
    } else {
        { attached: true, sent: false, filename: $name, size: ($text | str length) }
    }
}

# Send a message to ChatGPT
export def "chatgpt send" [
    message: string
    --model (-m): string  # Set model before sending (auto, instant, thinking)
    --new (-n)            # Start new temporary chat
]: nothing -> record {
    if $new {
        pw nav $"($BASE_URL)/?temporary-chat=true"
        pw wait-for "#prompt-textarea"
        sleep 500ms
    }

    if ($model | is-not-empty) {
        chatgpt set-model $model
    }

    # Use insert-text helper (handles newlines, escaping, large text)
    let result = (insert-text $message)
    if ($result | get -o error | is-not-empty) {
        error make { msg: ($result.error) }
    }

    # Wait for send button and click it
    sleep 100ms
    let send_result = (pw eval "(function() {
        const btn = document.querySelector('[data-testid=\"send-button\"]');
        if (!btn) return { error: 'send button not found' };
        if (btn.disabled) return { error: 'send button disabled' };
        btn.click();
        return { sent: true };
    })()").data.result

    if ($send_result | get -o error | is-not-empty) {
        error make { msg: ($send_result.error) }
    }

    { success: true, message: $message, model: (get-current-model) }
}

# Check if response is still in progress (thinking or streaming)
def is-generating []: nothing -> bool {
    let js = "(() => {
        // Check for thinking phase (5.2 Thinking model)
        const thinking = document.querySelector('.result-thinking');
        if (thinking) return true;

        // Check for streaming phase
        const stopBtn = document.querySelector('button[aria-label=\"Stop streaming\"]');
        if (stopBtn) return true;

        return false;
    })()"
    (pw eval $js).data.result
}

# Get count of assistant messages
def message-count []: nothing -> int {
    let js = "document.querySelectorAll(\"[data-message-author-role='assistant']\").length"
    (pw eval $js).data.result
}

# Wait for ChatGPT response to complete
export def "chatgpt wait" [
    --timeout (-t): int = 120000  # Timeout in ms
]: nothing -> record {
    let start = (date now)
    let initial_count = (message-count)
    let timeout_dur = ($timeout | into duration --unit ms)

    # Wait for streaming to start (stop button appears or message count increases)
    mut started = false
    for _ in 1..300 {
        if ((date now) - $start) > $timeout_dur { break }
        if (is-generating) or ((message-count) > $initial_count) {
            $started = true
            break
        }
        sleep 200ms
    }

    if not $started {
        return { complete: false, timeout: true, reason: "streaming never started" }
    }

    # Wait for streaming to complete (stop button disappears)
    for _ in 1..600 {
        if ((date now) - $start) > $timeout_dur {
            return { complete: false, timeout: true, reason: "streaming timeout" }
        }
        if not (is-generating) {
            let elapsed = ((date now) - $start) | format duration ms | str replace ' ms' '' | into int
            return { complete: true, elapsed: $elapsed }
        }
        sleep 300ms
    }

    { complete: false, timeout: true, reason: "loop exhausted" }
}

# Get the last response from ChatGPT
export def "chatgpt get-response" []: nothing -> string {
    let js = "(() => {
        const messages = document.querySelectorAll(\"[data-message-author-role='assistant']\");
        if (messages.length === 0) return null;
        const last = messages[messages.length - 1];
        return last.innerText;
    })()"
    (pw eval $js).data.result
}

# Send message and wait for response
export def "chatgpt ask" [
    message: string
    --model (-m): string
    --new (-n)
    --timeout (-t): int = 120000
]: nothing -> record {
    let initial_count = (message-count)
    chatgpt send $message --model=$model --new=$new
    let wait_result = (chatgpt wait --timeout=$timeout)
    let response = (chatgpt get-response)

    # Consider successful if we have a new response, even if wait timed out (fast responses)
    let has_new_response = (message-count) > $initial_count and ($response | is-not-empty)
    let success = $wait_result.complete or $has_new_response

    {
        success: $success
        message: $message
        response: $response
        elapsed_ms: ($wait_result | get -o elapsed)
    }
}
