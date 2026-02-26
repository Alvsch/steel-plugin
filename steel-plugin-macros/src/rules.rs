pub struct FnRules<'a> {
    pub name: &'a str,
    pub params: Option<&'a [&'a str]>,
    pub ret: Option<&'a str>,
    pub require_pub: bool,
}

impl Default for FnRules<'_> {
    fn default() -> Self {
        Self {
            name: "",
            params: None,
            ret: None,
            require_pub: true,
        }
    }
}
