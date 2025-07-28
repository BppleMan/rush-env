use crate::say_section;

#[derive(Debug, Clone)]
pub struct Section {
    pub width: usize,
    pub padding: usize,
}

impl Default for Section {
    fn default() -> Self {
        Section { width: 48, padding: 2 }
    }
}

impl Section {
    pub fn new(width: usize, padding: usize) -> Self {
        Section { width, padding }
    }

    pub fn say(&self, writer: &mut impl std::io::Write, content: impl AsRef<str>) -> std::io::Result<()> {
        say_section(writer, content.as_ref(), self.width, self.padding)
    }
}
