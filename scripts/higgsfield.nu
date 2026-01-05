# pw-sites.nu - Site-specific browser automation workflows
#
# Usage:
#   use pw.nu
#   use pw-sites.nu
#   higgsfield create-image "A dragon in a cyberpunk city"

use pw.nu

# Higgsfield AI image generation
export def "higgsfield create-image" [
    prompt: string
    --model (-m): string = "nano_banana_2"
    --wait-for-result (-w)
]: nothing -> record {
    pw nav $"https://higgsfield.ai/image/($model)"
    pw wait-for "textarea[name=prompt]"
    pw fill "textarea[name=prompt]" $prompt
    pw click "button:has-text('Generate')"
    
    if $wait_for_result {
        pw wait-for "[class*='generated'], [class*='complete']" -t 120000
    }
    
    { success: true, model: $model, prompt: $prompt, url: (pw url) }
}

# Higgsfield video generation  
export def "higgsfield create-video" [
    prompt: string
    --model (-m): string = "wan_2_6"
    --wait-for-result (-w)
]: nothing -> record {
    pw nav $"https://higgsfield.ai/create/video?model=($model)"
    pw wait-for "textarea[name=prompt]"
    pw fill "textarea[name=prompt]" $prompt
    pw click "button:has-text('Generate')"
    
    if $wait_for_result {
        pw wait-for "[class*='generated'], [class*='complete']" -t 300000
    }
    
    { success: true, model: $model, prompt: $prompt, url: (pw url) }
}
