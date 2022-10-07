use serde::{Deserialize, Serialize};
use tl::{HTMLTag, NodeHandle};

#[derive(Serialize, Deserialize, Debug)]
pub struct Substitution {
    pub teacher: String,
    pub lessons: Vec<Lesson>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lesson {
    pub lesson: u32,
    pub subject: String,
    pub class: String,
    pub classroom: String,
    pub info: Option<String>,
    #[serde(rename = "originalTeacher")]
    pub original_teacher: String,
    #[serde(rename = "originalSubject")]
    pub original_subject: String,
}

pub fn parse(html: String) -> Result<Vec<Substitution>, Box<dyn std::error::Error>> {
    let trimmed = html.replace("\t", "").replace("\n", "").replace("\r", "");

    let dom = tl::parse(&trimmed, tl::ParserOptions::default())?;
    let parser = dom.parser();

    // Get rows from teacher table
    let mut rows = dom
        .get_elements_by_class_name("nelogiran_seznam_nadomescanj")
        .collect::<Vec<NodeHandle>>()[1]
        .get(parser)
        .ok_or("Teacher table not found")?
        .as_tag()
        .ok_or("Failed to parse teacher table")?
        .query_selector(parser, "tr")
        .ok_or("Rows not found")?
        .collect::<Vec<NodeHandle>>();

    rows.remove(0);

    let mut substitutions: Vec<Substitution> = vec![];

    for row in rows {
        let mut children = row
            .get(parser)
            .ok_or("Failed to parse row")?
            .children()
            .ok_or("Row children not found")?
            .top()
            .iter()
            .filter_map(|c| c.get(parser)?.as_tag())
            .collect::<Vec<&HTMLTag>>();

        // Remove first (teacher) row if exists, for rowspan cells
        if children.len() == 5 {
            substitutions.push(Substitution {
                teacher: children[0].inner_text(parser).to_string(),
                lessons: vec![],
            });
            children.remove(0);
        }

        let sub = children[1]
            .children()
            .top()
            .iter()
            .filter_map(|c| {
                let node = c.get(parser)?;
                if let Some(tag) = node.as_tag() {
                    return Some(tag.inner_text(parser).to_string());
                }
                Some(node.as_raw()?.as_utf8_str().to_string())
            })
            .collect::<Vec<String>>();

        let original = sub[3].split(",").collect::<Vec<&str>>();

        let info = {
            let info_text = children[3].inner_text(parser).to_string();
            if info_text == "/" {
                None
            } else {
                Some(info_text)
            }
        };

        let lesson = Lesson {
            lesson: children[0]
                .inner_text(parser)
                .replace(".", "")
                .parse::<u32>()?,
            subject: sub[1].replace(",", "").trim().to_string(),
            class: sub[0].trim().to_string(),
            classroom: children[2].inner_text(parser).to_string(),
            info,
            original_teacher: original[0].replace("namesto", "").trim().to_string(),
            original_subject: original[1].trim().to_string(),
        };

        if let Some(sub) = substitutions.last_mut() {
            sub.lessons.push(lesson);
        }
    }

    Ok(substitutions)
}
