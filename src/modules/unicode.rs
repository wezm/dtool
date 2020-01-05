use clap::{SubCommand, Arg, ArgMatches};
use crate::modules::{Command, base, Case};
use std::char::EscapeUnicode;

static FORMAT_HELP: &str = "Format
<default>: \\u7c
html: &#x7c;
html_d: &#124;
rust: \\u{7c}";

pub fn commands<'a, 'b>() -> Vec<Command<'a, 'b>> {
	vec![
		Command {
			app: SubCommand::with_name("s2u").about("UTF-8 string to unicode")
				.arg(
					Arg::with_name("FORMAT")
						.long("format")
						.short("f").help(FORMAT_HELP)
						.takes_value(true)
						.required(false))
				.arg(
					Arg::with_name("INPUT")
						.required(false)
						.index(1)),
			f: s2u,
			cases: vec![
				Case {
					input: vec!["abc"].into_iter().map(Into::into).collect(),
					output: vec!["\\u61\\u62\\u63"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
				Case {
					input: vec!["-f", "html", "abc"].into_iter().map(Into::into).collect(),
					output: vec!["&#x61;&#x62;&#x63;"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
				Case {
					input: vec!["-f", "html_d", "abc"].into_iter().map(Into::into).collect(),
					output: vec!["&#97;&#98;&#99;"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
				Case {
					input: vec!["-f", "rust", "abc"].into_iter().map(Into::into).collect(),
					output: vec!["\\u{61}\\u{62}\\u{63}"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
			],
		},
		Command {
			app: SubCommand::with_name("u2s").about("Unicode to UTF-8 string")
				.arg(
					Arg::with_name("INPUT")
						.required(false)
						.index(1)),
			f: u2s,
			cases: vec![
				Case {
					input: vec!["\\u61\\u62\\u63"].into_iter().map(Into::into).collect(),
					output: vec!["abc"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
				Case {
					input: vec!["&#x61;&#x62;&#x63;"].into_iter().map(Into::into).collect(),
					output: vec!["abc"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
				Case {
					input: vec!["&#97;&#98;&#99;"].into_iter().map(Into::into).collect(),
					output: vec!["abc"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
				Case {
					input: vec!["\\u{61}\\u{62}\\u{63}"].into_iter().map(Into::into).collect(),
					output: vec!["abc"].into_iter().map(Into::into).collect(),
					is_example: true,
					is_test: true,
				},
			],
		},
	]
}

fn s2u(matches: &ArgMatches) -> Result<Vec<String>, String> {
	let input = base::input_string(matches)?;

	let format = match matches.value_of("FORMAT") {
		Some("html") => format_html,
		Some("html_d") => format_html_d,
		Some("rust") => format_rust,
		_ => format_default,
	};

	let result = input.chars().map(char::escape_unicode).map(format).collect::<Result<Vec<String>, String>>()?;

	let result = result.join("");

	Ok(vec![result])
}

fn u2s(matches: &ArgMatches) -> Result<Vec<String>, String> {
	let input = base::input_string(matches)?;

	let format = match input {
		_ if input.starts_with("\\u{") => "rust",
		_ if input.starts_with("&#x") => "html",
		_ if input.starts_with("&#") => "html_d",
		_ => "",
	};

	let result = match format {
		"html" => {
			input.split(";")
				.filter_map(from_html).collect::<Result<String, String>>()
		}
		"html_d" => {
			input.split(";")
				.filter_map(from_html_d).collect::<Result<String, String>>()
		}
		"rust" => {
			input.split("}")
				.filter_map(from_rust).collect::<Result<String, String>>()
		}
		_ => {
			input.split("\\u")
				.filter_map(from_default).collect::<Result<String, String>>()
		}
	}?;

	Ok(vec![result])
}

fn format_html(data: EscapeUnicode) -> Result<String, String> {
	Ok(data.map(|x| {
		match x {
			'\\' => '&',
			'u' => '#',
			'{' => 'x',
			'}' => ';',
			_ => x,
		}
	}).collect())
}

fn from_html(data: &str) -> Option<Result<char, String>> {
	if data.len() > 3 {
		let r = u32::from_str_radix(&data[3..], 16).map_err(|_| "Convert failed".to_string()).and_then(|x| {
			std::char::from_u32(x).ok_or("Convert failed".to_string())
		});
		Some(r)
	} else {
		None
	}
}

fn format_html_d(data: EscapeUnicode) -> Result<String, String> {
	let number = data.filter_map(|x| match x {
		'\\' | 'u' | '{' | '}' => None,
		_ => Some(x),
	}).collect::<String>();
	let number = u64::from_str_radix(&number, 16).map_err(|_| "Convert failed")?;

	Ok(format!("&#{};", number))
}

fn from_html_d(data: &str) -> Option<Result<char, String>> {
	if data.len() > 2 {
		let r = u32::from_str_radix(&data[2..], 10).map_err(|_| "Convert failed".to_string()).and_then(|x| {
			std::char::from_u32(x).ok_or("Convert failed".to_string())
		});
		Some(r)
	} else {
		None
	}
}

fn format_rust(data: EscapeUnicode) -> Result<String, String> {
	Ok(data.collect())
}

fn from_rust(data: &str) -> Option<Result<char, String>> {
	if data.len() > 3 {
		let r = u32::from_str_radix(&data[3..], 16).map_err(|_| "Convert failed".to_string()).and_then(|x| {
			std::char::from_u32(x).ok_or("Convert failed".to_string())
		});
		Some(r)
	} else {
		None
	}
}

fn format_default(data: EscapeUnicode) -> Result<String, String> {
	Ok(data.filter(|x| x != &'{' && x != &'}').collect())
}

fn from_default(data: &str) -> Option<Result<char, String>> {
	if data.len() > 0 {
		let r = u32::from_str_radix(&data, 16).map_err(|_| "Convert failed".to_string()).and_then(|x| {
			std::char::from_u32(x).ok_or("Convert failed".to_string())
		});
		Some(r)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::modules::base::test::test_commands;

	#[test]
	fn test_cases() {
		test_commands(&commands());
	}

}
