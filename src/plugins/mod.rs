pub mod memory;

pub trait Plugin {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn sample(&mut self) -> Output;
    fn flags(&self) -> Vec<Flag> {
        Vec::new()
    }
    fn set_flag(&mut self, _id: &str, _on: bool) {}
}

#[derive(Default)]
pub struct Output {
    pub segment: Option<String>,
    pub tooltip_lines: Vec<String>,
    pub menu_items: Option<Vec<Item>>,
}

#[derive(PartialEq)]
pub struct Item {
    pub text: String,
    pub badge: Option<String>,
}

pub struct Flag {
    pub id: &'static str,
    pub label: &'static str,
    pub on: bool,
}

pub fn all() -> Vec<Box<dyn Plugin>> {
    vec![Box::new(memory::Memory::new())]
}
