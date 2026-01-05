# pw.nu - Nushell module for pw browser automation
#
# Usage:
#   use pw.nu
#   pw nav "https://example.com"
#   pw text "h1"
#   pw click "button.submit"
#
# Or import specific commands:
#   use pw.nu [nav text click fill]

# Run pw command and parse JSON output
# Uses --wrapped to pass flags through without parsing
def --wrapped pw-run [...args: string]: nothing -> record {
    let result = (^pw -f json ...$args | complete)
    if $result.exit_code != 0 {
        let parsed = try { $result.stdout | from json } catch { null }
        if $parsed != null and ($parsed.error? != null) {
            error make { msg: $parsed.error.message }
        } else {
            error make { msg: ($result.stderr | str trim) }
        }
    }
    $result.stdout | from json
}

# Navigate to URL
export def nav [
    url: string  # URL to navigate to
]: nothing -> record {
    pw-run navigate $url
}

# Get text content of element
export def text [
    selector: string  # CSS selector
]: nothing -> record {
    pw-run text -s $selector
}

# Get HTML content of element
export def html [
    selector: string = "html"  # CSS selector
]: nothing -> record {
    pw-run html -s $selector
}

# Click element
export def click [
    selector: string  # CSS selector
]: nothing -> record {
    pw-run click -s $selector
}

# Fill input field (works with React)
export def fill [
    selector: string  # CSS selector
    value: string     # Text to fill
]: nothing -> record {
    pw-run fill $value -s $selector
}

# Take screenshot
export def screenshot [
    --output (-o): string = "screenshot.png"  # Output file
    --full-page (-f)  # Capture full scrollable page
]: nothing -> record {
    if $full_page {
        pw-run screenshot -o $output --full-page
    } else {
        pw-run screenshot -o $output
    }
}

# Evaluate JavaScript
export def eval [
    expression: string  # JavaScript expression
]: nothing -> record {
    pw-run eval $expression
}

# Wait for condition
export def wait [
    condition: string     # Selector or condition
    timeout_ms?: int      # Timeout in milliseconds
]: nothing -> record {
    if $timeout_ms != null {
        pw-run wait $condition --timeout-ms ($timeout_ms | into string)
    } else {
        pw-run wait $condition
    }
}

# List interactive elements
export def elements [
    --wait (-w)  # Wait for elements to appear
]: nothing -> record {
    if $wait {
        pw-run elements --wait
    } else {
        pw-run elements
    }
}

# Connect to CDP endpoint
export def connect [
    endpoint?: string  # CDP WebSocket URL
    --clear (-c)       # Clear saved endpoint
]: nothing -> record {
    if $clear {
        pw-run connect --clear
    } else if $endpoint != null {
        pw-run connect $endpoint
    } else {
        pw-run connect
    }
}

# List browser tabs
export def tabs []: nothing -> record {
    pw-run tabs list
}

# Switch to tab
export def "tabs switch" [
    target: string  # Tab index or URL pattern
]: nothing -> record {
    pw-run tabs switch $target
}

# Extract readable content
export def read [
    url?: string  # URL to read (uses current page if omitted)
]: nothing -> record {
    if $url != null {
        pw-run read $url
    } else {
        pw-run read
    }
}

# =============================================================================
# Higher-level workflow helpers
# =============================================================================

# Get text content directly as string
export def text-only [
    selector: string  # CSS selector
]: nothing -> string {
    (text $selector).data.text
}

# Check if element exists
export def exists [
    selector: string  # CSS selector
]: nothing -> bool {
    try {
        let result = (pw-run eval $"document.querySelector\('($selector)'\) !== null")
        $result.data.result == true
    } catch {
        false
    }
}

# Wait for element and return when ready
export def wait-for [
    selector: string      # CSS selector
    --timeout (-t): int = 30000  # Timeout in ms
]: nothing -> bool {
    try {
        pw-run wait $selector --timeout-ms ($timeout | into string)
        true
    } catch {
        false
    }
}

# Fill form fields from record
export def fill-form [
    fields: record  # Field name -> value mapping
]: nothing -> list {
    $fields | items {|name, value|
        let selector = $"[name='($name)'], #($name), input[placeholder*='($name)' i]"
        try {
            fill $selector $value
            { field: $name, status: "ok" }
        } catch {|e|
            { field: $name, status: "error", error: $e.msg }
        }
    }
}

# Get current page URL
export def url []: nothing -> string {
    (pw-run eval "window.location.href").data.result
}

# Get current page title  
export def title []: nothing -> string {
    (pw-run eval "document.title").data.result
}

# =============================================================================
# Site-specific workflows
# =============================================================================

# Higgsfield image generation
export def "higgsfield create-image" [
    prompt: string                    # Image generation prompt
    --model (-m): string = "nano_banana_2"  # Model to use
    --wait-for-result (-w)            # Wait for generation to complete
]: nothing -> record {
    # Navigate to the model page
    let nav_result = (nav $"https://higgsfield.ai/image/($model)")
    
    # Wait for prompt input to be ready
    wait-for "textarea[name=prompt]"
    
    # Fill the prompt
    fill "textarea[name=prompt]" $prompt
    
    # Click generate
    click "button:has-text('Generate')"
    
    if $wait_for_result {
        # Wait for generation (look for result or progress completion)
        wait-for "[class*='generated'], [class*='result'], [class*='complete']" -t 120000
    }
    
    {
        success: true
        model: $model
        prompt: $prompt
        url: (url)
    }
}

# Generic site workflow runner
export def workflow [
    steps: list  # List of step records
]: nothing -> list {
    $steps | each {|step|
        let step_type = ($step | columns | first)
        match $step_type {
            "nav" => { nav ($step | get nav) }
            "navigate" => { nav ($step | get navigate) }
            "click" => { click ($step | get click) }
            "fill" => { 
                let f = ($step | get fill)
                fill $f.selector $f.value 
            }
            "text" => { text ($step | get text) }
            "wait" => { wait ($step | get wait) }
            "eval" => { eval ($step | get eval) }
            _ => { error make { msg: $"Unknown step type: ($step_type)" } }
        }
    }
}
