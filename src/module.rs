#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub label: String,
    pub value: String,
}

impl Field {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Field {
            label: label.into(),
            value: value.into(),
        }
    }

    /// A continuation line, aligned under the first field's value.
    pub fn cont(value: impl Into<String>) -> Self {
        Field {
            label: String::new(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleOutput {
    pub name: String,
    pub fields: Vec<Field>,
}

impl ModuleOutput {
    pub fn new(name: &str, value: impl Into<String>) -> Self {
        ModuleOutput {
            name: name.to_string(),
            fields: vec![Field::new("", value)],
        }
    }

    pub fn labeled(name: &str, label: impl Into<String>, value: impl Into<String>) -> Self {
        ModuleOutput {
            name: name.to_string(),
            fields: vec![Field::new(label, value)],
        }
    }

    pub fn with(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(Field::new(label, value));
        self
    }

    pub fn line(mut self, value: impl Into<String>) -> Self {
        self.fields.push(Field::cont(value));
        self
    }

    /// Several values under one key: the first on the key line, the rest indented below.
    pub fn multi(name: &str, values: impl IntoIterator<Item = String>) -> Self {
        let mut fields: Vec<Field> = values.into_iter().map(Field::cont).collect();
        if fields.is_empty() {
            fields.push(Field::cont(""));
        }
        ModuleOutput {
            name: name.to_string(),
            fields,
        }
    }

    /// Value of the first field, for modules used as a single token (e.g. OS id).
    pub fn first(&self) -> &str {
        self.fields.first().map(|f| f.value.as_str()).unwrap_or("")
    }
}

pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;
    /// Stable id used by --module / config, e.g. "os".
    fn id(&self) -> &'static str;
    fn run(&self) -> Option<ModuleOutput>;
}

pub type BoxedModule = Box<dyn Module>;
