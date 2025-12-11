//! # SSR Module - The SEO Inflator
//!
//! Converts binary templates + state into pure HTML for GoogleBot
//!
//! **Performance Target:** ~1ms per page (faster than Next.js SSR)



/// SEO Inflator - Converts binary template + data into HTML string
/// Inflate a template string with data
///
/// # Performance
/// - No DOM creation (unlike Next.js)
/// - Pure string replacement
/// - ~1ms per page
pub fn inflate(template_html: &str, slots: &[(String, String)]) -> String {
    let mut result = template_html.to_string();

    // Replace all slots
    for (slot_marker, value) in slots {
        let marker = format!("<!--{}-->", slot_marker);
        result = result.replace(&marker, value);
    }

    result
}

/// Inflate full page with DOCTYPE and metadata
pub fn inflate_page(
    template_html: &str,
    slots: &[(String, String)],
    title: &str,
    meta: &[(String, String)],
) -> String {
    let body = inflate(template_html, slots);

    let mut html = String::from("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str(&format!("    <meta charset=\"UTF-8\">\n"));
    html.push_str(&format!(
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n"
    ));
    html.push_str(&format!("    <title>{}</title>\n", title));

    // Add meta tags
    for (name, content) in meta {
        html.push_str(&format!("    <meta name=\"{}\" content=\"{}\">\n", name, content));
    }

    html.push_str("</head>\n<body>\n");
    html.push_str(&body);
    html.push_str("\n</body>\n</html>");

    html
}

/// Detect if User-Agent is a search engine bot
pub fn is_bot(user_agent: &str) -> bool {
    let ua_lower = user_agent.to_lowercase();

    ua_lower.contains("googlebot")
        || ua_lower.contains("bingbot")
        || ua_lower.contains("slurp") // Yahoo
        || ua_lower.contains("duckduckbot")
        || ua_lower.contains("baiduspider")
        || ua_lower.contains("yandexbot")
        || ua_lower.contains("facebookexternalhit")
        || ua_lower.contains("twitterbot")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_inflation() {
        let template = "<div><!--SLOT_0--></div>";
        let result = inflate(template, &[("SLOT_0".to_string(), "Hello World".to_string())]);
        assert_eq!(result, "<div>Hello World</div>");
    }

    #[test]
    fn test_multiple_slots() {
        let template = "<div><!--SLOT_0--> and <!--SLOT_1--></div>";
        let result = inflate(
            template,
            &[
                ("SLOT_0".to_string(), "Hello".to_string()),
                ("SLOT_1".to_string(), "World".to_string()),
            ],
        );
        assert_eq!(result, "<div>Hello and World</div>");
    }

    #[test]
    fn test_bot_detection() {
        assert!(is_bot("Mozilla/5.0 (compatible; Googlebot/2.1)"));
        assert!(is_bot("Mozilla/5.0 (compatible; bingbot/2.0)"));
        assert!(!is_bot("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0"));
    }
}

/// Detect if User-Agent is a search engine bot
pub fn is_bot(user_agent: &str) -> bool {
    let ua_lower = user_agent.to_lowercase();

    ua_lower.contains("googlebot")
        || ua_lower.contains("bingbot")
        || ua_lower.contains("slurp") // Yahoo
        || ua_lower.contains("duckduckbot")
        || ua_lower.contains("baiduspider")
        || ua_lower.contains("yandexbot")
        || ua_lower.contains("facebookexternalhit")
        || ua_lower.contains("twitterbot")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_inflation() {
        let mut inflator = SsrInflator::new();

        inflator.register_template(1, "<div><!--SLOT_0--></div>".to_string());

        let result = inflator.inflate(1, &[("SLOT_0".to_string(), "Hello World".to_string())]);

        assert_eq!(result, Some("<div>Hello World</div>".to_string()));
    }

    #[test]
    fn test_multiple_slots() {
        let mut inflator = SsrInflator::new();

        inflator.register_template(2, "<div><!--SLOT_0--> and <!--SLOT_1--></div>".to_string());

        let result = inflator.inflate(
            2,
            &[
                ("SLOT_0".to_string(), "Hello".to_string()),
                ("SLOT_1".to_string(), "World".to_string()),
            ],
        );

        assert_eq!(result, Some("<div>Hello and World</div>".to_string()));
    }

    #[test]
    fn test_bot_detection() {
        assert!(is_bot("Mozilla/5.0 (compatible; Googlebot/2.1)"));
        assert!(is_bot("Mozilla/5.0 (compatible; bingbot/2.0)"));
        assert!(!is_bot("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0"));
    }

    #[test]
    fn test_full_page_inflation() {
        let mut inflator = SsrInflator::new();

        inflator.register_template(1, "<h1><!--SLOT_0--></h1>".to_string());

        let result = inflator.inflate_page(
            1,
            &[("SLOT_0".to_string(), "Welcome".to_string())],
            "My Page",
            &[
                ("description".to_string(), "Test page".to_string()),
                ("keywords".to_string(), "test, dx-www".to_string()),
            ],
        );

        assert!(result.is_some());
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>My Page</title>"));
        assert!(html.contains("<h1>Welcome</h1>"));
        assert!(html.contains("description"));
    }
}
