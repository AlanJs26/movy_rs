use crate::error::{Error, Result};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Regex { content: String, flags: Vec<String> },
}

impl Value {
    pub fn match_in(&self, content: String) -> Result<bool> {
        match self {
            Self::String(value) => Ok(content.contains(value)),
            Self::Boolean(_) => Err(Error::Generic(
                "Cannot match string against boolean".to_string(),
            )),
            Self::Number(value) => Ok(content.contains(value.to_string().as_str())),
            Self::Regex {
                content: _,
                flags: _,
            } => {
                let re = self.build_regex()?;

                Ok(re.is_match(&content))
            }
        }
    }

    fn build_regex(&self) -> Result<Regex> {
        if let Self::Regex { content, flags } = self {
            let mut regex_string = String::from("");
            for flag in flags {
                regex_string.push_str(format!("(?{})", flag).as_str());
            }
            regex_string.push_str(content);

            return Ok(Regex::new(&regex_string)?);
        }

        return Err(Error::Generic(
            "Cannot build regex from invalid Value".to_string(),
        ));
    }
}
