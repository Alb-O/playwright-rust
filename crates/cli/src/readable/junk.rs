//! Junk-line filtering shared by text and markdown renderers.

use crate::readable::config::clutter;

pub(crate) fn is_junk_line(line: &str) -> bool {
	let mut remaining = line.to_string();
	for pattern in &clutter().junk_text.exact {
		let pattern_lower = pattern.to_lowercase();
		let mut result = String::new();
		let mut remaining_lower = remaining.to_lowercase();
		while let Some(pos) = remaining_lower.find(&pattern_lower) {
			result.push_str(&remaining[..pos]);
			remaining = remaining[pos + pattern.len()..].to_string();
			remaining_lower = remaining.to_lowercase();
		}
		result.push_str(&remaining);
		remaining = result;
	}

	remaining.trim().chars().all(|c| c.is_whitespace() || "/-•·|:".contains(c))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn detects_pure_junk_lines() {
		assert!(is_junk_line("NaN"));
		assert!(is_junk_line("NaN / NaN"));
		assert!(is_junk_line("undefined"));
		assert!(is_junk_line("[object Object]"));
	}

	#[test]
	fn keeps_real_content_lines() {
		assert!(!is_junk_line("The value is NaN due to division"));
		assert!(!is_junk_line("undefined behavior in C++"));
		assert!(!is_junk_line("Hello World"));
		assert!(!is_junk_line("10:30 AM"));
	}
}
