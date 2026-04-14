use cssparser::{Parser, ParserInput, Token};

use crate::types::{PageMargins, PageSize};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageRuleInfo {
    pub size: Option<PageSize>,
    pub landscape: bool,
    pub margins: Option<PageMargins>,
}

pub fn parse_page_rules(css: &str) -> PageRuleInfo {
    let mut info = PageRuleInfo::default();
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);

    while !parser.is_exhausted() {
        let result = parser.try_parse(|p| -> Result<(), cssparser::ParseError<'_, ()>> {
            let token = p.next()?.clone();
            if let Token::AtKeyword(ref name) = token {
                if name.eq_ignore_ascii_case("page") {
                    // Skip optional page name/pseudo
                    while let Ok(t) = p.next() {
                        if matches!(t, Token::CurlyBracketBlock) {
                            break;
                        }
                    }
                    p.parse_nested_block(|p| {
                        parse_page_declarations(p, &mut info);
                        Ok(())
                    })?;
                    return Ok(());
                }
            }
            Ok(())
        });
        if result.is_err() {
            let _ = parser.next();
        }
    }

    info
}

fn parse_page_declarations(parser: &mut Parser<'_, '_>, info: &mut PageRuleInfo) {
    while !parser.is_exhausted() {
        let result = parser.try_parse(|p| -> Result<(), cssparser::ParseError<'_, ()>> {
            let name = p.expect_ident()?.clone();
            p.expect_colon()?;

            match name.as_ref() {
                "size" => parse_size(p, info),
                "margin" => parse_margin_shorthand(p, info),
                "margin-top" => {
                    if let Some(v) = parse_length_pt(p) {
                        info.margins.get_or_insert_with(PageMargins::default).top = v;
                    }
                }
                "margin-right" => {
                    if let Some(v) = parse_length_pt(p) {
                        info.margins.get_or_insert_with(PageMargins::default).right = v;
                    }
                }
                "margin-bottom" => {
                    if let Some(v) = parse_length_pt(p) {
                        info.margins.get_or_insert_with(PageMargins::default).bottom = v;
                    }
                }
                "margin-left" => {
                    if let Some(v) = parse_length_pt(p) {
                        info.margins.get_or_insert_with(PageMargins::default).left = v;
                    }
                }
                _ => {}
            }

            let _ = p.try_parse(|p| p.expect_semicolon());
            Ok(())
        });
        if result.is_err() {
            let _ = parser.next();
        }
    }
}

fn parse_size(parser: &mut Parser<'_, '_>, info: &mut PageRuleInfo) {
    let mut values: Vec<String> = Vec::new();

    while let Ok(token) = parser.next_including_whitespace() {
        match token {
            Token::Ident(name) => values.push(name.to_string()),
            Token::Semicolon | Token::CloseCurlyBracket => break,
            Token::WhiteSpace(_) => continue,
            _ => break,
        }
    }

    for val in &values {
        match val.to_uppercase().as_str() {
            "A3" => info.size = Some(PageSize::A3),
            "A4" => info.size = Some(PageSize::A4),
            "A5" => info.size = Some(PageSize::A5),
            "LETTER" => info.size = Some(PageSize::Letter),
            "LEGAL" => info.size = Some(PageSize::Legal),
            "TABLOID" | "LEDGER" => info.size = Some(PageSize::Tabloid),
            "LANDSCAPE" => info.landscape = true,
            "PORTRAIT" => info.landscape = false,
            _ => {}
        }
    }
}

fn parse_margin_shorthand(parser: &mut Parser<'_, '_>, info: &mut PageRuleInfo) {
    let mut values = Vec::new();
    while let Some(v) = parse_length_pt(parser) {
        values.push(v);
        if values.len() >= 4 {
            break;
        }
    }

    let margins = match values.len() {
        1 => PageMargins {
            top: values[0],
            right: values[0],
            bottom: values[0],
            left: values[0],
        },
        2 => PageMargins {
            top: values[0],
            right: values[1],
            bottom: values[0],
            left: values[1],
        },
        3 => PageMargins {
            top: values[0],
            right: values[1],
            bottom: values[2],
            left: values[1],
        },
        4 => PageMargins {
            top: values[0],
            right: values[1],
            bottom: values[2],
            left: values[3],
        },
        _ => return,
    };

    info.margins = Some(margins);
}

fn parse_length_pt(parser: &mut Parser<'_, '_>) -> Option<f32> {
    parser
        .try_parse(|p| {
            let token = p.next()?.clone();
            match &token {
                Token::Dimension { value, unit, .. } => {
                    let pt = match unit.as_ref() {
                        "mm" => value * 2.8346457,
                        "cm" => value * 28.346457,
                        "in" => value * 72.0,
                        "px" => value * 0.75,
                        "pt" => *value,
                        "pc" => value * 12.0,
                        _ => return Err(p.new_custom_error::<(), ()>(())),
                    };
                    Ok(pt)
                }
                Token::Number { value, .. } if *value == 0.0 => Ok(0.0),
                _ => Err(p.new_custom_error::<(), ()>(())),
            }
        })
        .ok()
}

pub fn extract_style_css(html: &str) -> String {
    let mut result = String::new();
    let mut remaining = html;
    while let Some(start_tag_pos) = remaining.find("<style") {
        let after_tag = &remaining[start_tag_pos..];
        if let Some(tag_end) = after_tag.find('>') {
            let content_start = start_tag_pos + tag_end + 1;
            let content_area = &remaining[content_start..];
            if let Some(close_pos) = content_area.find("</style>") {
                result.push_str(&content_area[..close_pos]);
                result.push('\n');
                remaining = &content_area[close_pos + 8..];
                continue;
            }
        }
        break;
    }
    result
}
