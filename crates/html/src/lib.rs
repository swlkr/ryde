use std::{borrow::Cow, collections::HashSet, fmt::Display, io::Write};

extern crate self as html;

fn escape<'a, S: Into<Cow<'a, str>>>(input: S) -> Cow<'a, str> {
    let input = input.into();
    fn needs_escaping(c: char) -> bool {
        c == '<' || c == '>' || c == '&' || c == '"' || c == '\''
    }

    if let Some(first) = input.find(needs_escaping) {
        let mut output = String::from(&input[0..first]);
        output.reserve(input.len() - first);
        let rest = input[first..].chars();
        for c in rest {
            match c {
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                '&' => output.push_str("&amp;"),
                '"' => output.push_str("&quot;"),
                '\'' => output.push_str("&#39;"),
                _ => output.push(c),
            }
        }
        Cow::Owned(output)
    } else {
        input
    }
}

pub struct Element {
    name: &'static str,
    attrs: Vec<u8>,
    children: Option<Box<dyn Render>>,
    class: String,
    css: Vec<String>,
}

macro_rules! impl_attr {
    ($ident:ident) => {
        pub fn $ident(self, value: impl Display) -> Self {
            self.attr(stringify!($ident), value)
        }
    };

    ($ident:ident, $name:expr) => {
        pub fn $ident(self, value: impl Display) -> Self {
            self.attr($name, value)
        }
    };
}

macro_rules! impl_bool_attr {
    ($ident:ident) => {
        pub fn $ident(self) -> Self {
            self.bool_attr(stringify!($ident))
        }
    };
}

impl Element {
    fn new(name: &'static str, children: Option<Box<dyn Render>>) -> Element {
        Element {
            name,
            attrs: vec![],
            children,
            class: "".into(),
            css: vec![],
        }
    }

    pub fn attr(mut self, name: &'static str, value: impl Display) -> Self {
        if !self.attrs.is_empty() {
            self.attrs
                .write(b" ")
                .expect("attr failed to write to buffer");
        }
        self.attrs
            .write_fmt(format_args!("{}", name))
            .expect("attr failed to write to buffer");
        self.attrs
            .write(b"=\"")
            .expect("attr failed to write to buffer");
        self.attrs
            .write_fmt(format_args!("{}", escape(value.to_string())))
            .expect("attr failed to write to buffer");
        self.attrs
            .write(b"\"")
            .expect("attr failed to write to buffer");

        self
    }

    pub fn bool_attr(mut self, name: &'static str) -> Self {
        if !self.attrs.is_empty() {
            self.attrs
                .write(b" ")
                .expect("bool_attr failed to write to buffer");
        }
        self.attrs
            .write_fmt(format_args!("{}", name))
            .expect("bool_attr failed to write to buffer");

        self
    }

    #[deprecated(since = "0.1.1", note = "Please use type_ instead")]
    pub fn r#type(self, value: impl Display) -> Self {
        self.attr("type", value)
    }

    #[deprecated(since = "0.1.1", note = "Please use for_ instead")]
    pub fn r#for(self, value: impl Display) -> Self {
        self.attr("for", value)
    }

    pub fn css(mut self, value: (impl Display, Vec<&str>)) -> Self {
        self.css.extend(value.1.into_iter().map(|x| x.to_string()));
        self.class(value.0)
    }

    pub fn class(mut self, value: impl Display) -> Self {
        if self.class.is_empty() {
            self.class = value.to_string();
        } else {
            self.class.push(' ');
            self.class.push_str(&value.to_string());
        }
        self
    }

    pub fn replace(self, value: impl Display) -> Self {
        self.attr("x-replace", value)
    }

    impl_attr!(id);
    impl_attr!(charset);
    impl_attr!(content);
    impl_attr!(name);
    impl_attr!(href);
    impl_attr!(rel);
    impl_attr!(target);
    impl_attr!(src);
    impl_attr!(integrity);
    impl_attr!(crossorigin);
    impl_attr!(role);
    impl_attr!(method);
    impl_attr!(action);
    impl_attr!(placeholder);
    impl_attr!(value);
    impl_attr!(rows);
    impl_attr!(alt);
    impl_attr!(style);
    impl_attr!(onclick);
    impl_attr!(placement);
    impl_attr!(toggle);
    impl_attr!(scope);
    impl_attr!(title);
    impl_attr!(lang);
    impl_attr!(type_, "type");
    impl_attr!(for_, "for");
    impl_attr!(aria_controls, "aria-controls");
    impl_attr!(aria_expanded, "aria-expanded");
    impl_attr!(aria_label, "aria-label");
    impl_attr!(aria_haspopup, "aria-haspopup");
    impl_attr!(aria_labelledby, "aria-labelledby");
    impl_attr!(aria_current, "aria-current");
    impl_bool_attr!(defer);
    impl_bool_attr!(checked);
    impl_bool_attr!(enabled);
    impl_bool_attr!(disabled);
}

pub trait Render {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()>;
    fn styles(&self, _styles: &mut HashSet<String>) -> std::io::Result<()> {
        Ok(())
    }
}

impl Render for Element {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        let name_bytes = self.name.as_bytes();
        buffer.write(b"<")?;
        buffer.write(name_bytes)?;
        if !self.attrs.is_empty() {
            buffer.write(b" ")?;
            buffer.write(&self.attrs)?;
        }
        if !self.class.is_empty() {
            buffer.write(b" ")?;
            buffer.write_fmt(format_args!("class=\"{}\"", self.class))?;
        }
        buffer.write(b">")?;
        match &self.children {
            Some(children) => {
                children.render(buffer)?;
                buffer.write(b"</")?;
                buffer.write(name_bytes)?;
                buffer.write(b">")?;
            }
            None => {}
        };

        Ok(())
    }

    fn styles(&self, styles: &mut HashSet<String>) -> std::io::Result<()> {
        if !self.css.is_empty() {
            styles.extend(self.css.clone().into_iter().collect::<HashSet<String>>());
        }
        match &self.children {
            Some(children) => {
                children.styles(styles)?;
            }
            None => {}
        };
        Ok(())
    }
}

pub struct Raw(pub String);

impl Render for Raw {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        buffer.write_fmt(format_args!("{}", self.0))?;

        Ok(())
    }
}

pub fn danger(html: impl Display) -> Raw {
    Raw(html.to_string())
}

impl Render for String {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        buffer.write_fmt(format_args!("{}", escape(self)))?;

        Ok(())
    }
}

impl Render for &String {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        buffer.write_fmt(format_args!("{}", escape(*self)))?;

        Ok(())
    }
}

impl<'a> Render for &'a str {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        buffer.write_fmt(format_args!("{}", escape(*self)))?;

        Ok(())
    }
}

impl Render for () {
    fn render(&self, _buffer: &mut Vec<u8>) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T> Render for Vec<T>
where
    T: Render,
{
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        for t in self {
            t.render(buffer)?;
        }

        Ok(())
    }

    fn styles(&self, buffer: &mut HashSet<String>) -> std::io::Result<()> {
        for t in self {
            t.styles(buffer)?;
        }

        Ok(())
    }
}

macro_rules! impl_render_tuple {
    ($max:expr) => {
        seq_macro::seq!(N in 0..=$max {
            impl<#(T~N,)*> Render for (#(T~N,)*)
            where
                #(T~N: Render,)*
            {
                fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
                    #(self.N.render(buffer)?;)*

                    Ok(())
                }

                fn styles(&self, buffer: &mut HashSet<String>) -> std::io::Result<()> {
                    #(self.N.styles(buffer)?;)*

                    Ok(())
                }
            }
        });
    };
}

seq_macro::seq!(N in 0..=31 {
    impl_render_tuple!(N);
});

pub fn doctype() -> Element {
    Element::new("!DOCTYPE html", None)
}

pub fn render(renderable: impl Render + 'static) -> String {
    let mut v: Vec<u8> = vec![];
    renderable.render(&mut v).expect("Failed to render html");
    String::from_utf8_lossy(&v).into()
}

pub fn styles(renderable: &impl Render) -> String {
    let mut styles: HashSet<String> = HashSet::new();
    renderable
        .styles(&mut styles)
        .expect("Failed to style html");
    let mut other_styles = styles
        .iter()
        .filter(|s| !s.starts_with("@media"))
        .collect::<Vec<_>>();
    let media_styles = styles
        .iter()
        .filter(|s| s.starts_with("@media"))
        .collect::<Vec<_>>();
    other_styles.extend(media_styles);
    other_styles
        .into_iter()
        .map(|s| s.clone())
        .collect::<Vec<_>>()
        .join("")
}

macro_rules! impl_render_num {
    ($t:ty) => {
        impl Render for $t {
            fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
                buffer.write_fmt(format_args!("{}", &self))?;
                Ok(())
            }
        }
    };
}

impl_render_num!(u8);
impl_render_num!(u16);
impl_render_num!(f64);
impl_render_num!(f32);
impl_render_num!(i64);
impl_render_num!(u64);
impl_render_num!(i32);
impl_render_num!(u32);
impl_render_num!(usize);
impl_render_num!(isize);

pub fn element(name: &'static str, children: impl Render + 'static) -> Element {
    Element::new(name, Some(Box::new(children)))
}

pub fn self_closing_element(name: &'static str) -> Element {
    Element::new(name, None)
}

pub fn anon_element(children: impl Render + 'static) -> Element {
    Element::new("", Some(Box::new(children)))
}

macro_rules! impl_element {
    ($ident:ident) => {
        pub fn $ident(child: impl Render + 'static) -> Element {
            Element::new(stringify!($ident), Some(Box::new(child)))
        }
    };
}

macro_rules! impl_void_element {
    ($ident:ident) => {
        pub fn $ident() -> Element {
            Element::new(stringify!($ident), None)
        }
    };
}

impl_element!(html);
impl_element!(head);
impl_element!(title);
impl_element!(body);
impl_element!(div);
impl_element!(section);
impl_element!(style);
impl_element!(h1);
impl_element!(h2);
impl_element!(h3);
impl_element!(h4);
impl_element!(h5);
impl_element!(li);
impl_element!(ul);
impl_element!(ol);
impl_element!(p);
impl_element!(span);
impl_element!(b);
impl_element!(i);
impl_element!(u);
impl_element!(tt);
impl_element!(string);
impl_element!(pre);
impl_element!(script);
impl_element!(main);
impl_element!(nav);
impl_element!(a);
impl_element!(form);
impl_element!(button);
impl_element!(blockquote);
impl_element!(footer);
impl_element!(wrapper);
impl_element!(label);
impl_element!(table);
impl_element!(thead);
impl_element!(th);
impl_element!(tr);
impl_element!(td);
impl_element!(tbody);
impl_element!(textarea);
impl_element!(datalist);
impl_element!(option);

impl_void_element!(area);
impl_void_element!(base);
impl_void_element!(br);
impl_void_element!(col);
impl_void_element!(embed);
impl_void_element!(hr);
impl_void_element!(img);
impl_void_element!(input);
impl_void_element!(link);
impl_void_element!(meta);
impl_void_element!(param);
impl_void_element!(source);
impl_void_element!(track);
impl_void_element!(wbr);

#[cfg(test)]
mod tests {
    use html::*;

    #[test]
    fn it_works() {
        let html = render((doctype(), html((head(()), body(())))));
        assert_eq!(
            "<!DOCTYPE html><html><head></head><body></body></html>",
            html
        );
    }

    #[test]
    fn it_works_with_numbers() {
        let html = render((doctype(), html((head(()), body(0)))));
        assert_eq!(
            "<!DOCTYPE html><html><head></head><body>0</body></html>",
            html
        );
    }

    #[test]
    fn it_escapes_correctly() {
        let html = render((doctype(), html((head(()), body("<div />")))));
        assert_eq!(
            html,
            "<!DOCTYPE html><html><head></head><body>&lt;div /&gt;</body></html>",
        );
    }

    #[test]
    fn it_escapes_more() {
        let html = render((
            doctype(),
            html((head(()), body("<script>alert('hello')</script>"))),
        ));
        assert_eq!(
            html,
            "<!DOCTYPE html><html><head></head><body>&lt;script&gt;alert(&#39;hello&#39;)&lt;/script&gt;</body></html>",
        );
    }

    #[test]
    fn it_renders_attributes() {
        let html = render((doctype(), html((head(()), body(div("hello").id("hello"))))));
        assert_eq!(
            "<!DOCTYPE html><html><head></head><body><div id=\"hello\">hello</div></body></html>",
            html
        );
    }

    #[test]
    fn it_renders_custom_self_closing_elements() {
        fn hx_close() -> Element {
            self_closing_element("hx-close")
        }
        let html = render(hx_close().id("id"));
        assert_eq!("<hx-close id=\"id\">", html);
    }

    #[test]
    fn readme_works() {
        use html::*;

        fn render_to_string(element: Element) -> String {
            render((
                doctype(),
                html((
                    head((title("title"), meta().charset("utf-8"))),
                    body(element),
                )),
            ))
        }

        assert_eq!(
        render_to_string(div("html")),
        "<!DOCTYPE html><html><head><title>title</title><meta charset=\"utf-8\"></head><body><div>html</div></body></html>"
      )
    }

    #[test]
    fn max_tuples_works() {
        let elements = seq_macro::seq!(N in 0..=31 {
            (#(br().id(N),)*)
        });

        assert_eq!(render(elements),
            "<br id=\"0\"><br id=\"1\"><br id=\"2\"><br id=\"3\"><br id=\"4\"><br id=\"5\"><br id=\"6\"><br id=\"7\"><br id=\"8\"><br id=\"9\"><br id=\"10\"><br id=\"11\"><br id=\"12\"><br id=\"13\"><br id=\"14\"><br id=\"15\"><br id=\"16\"><br id=\"17\"><br id=\"18\"><br id=\"19\"><br id=\"20\"><br id=\"21\"><br id=\"22\"><br id=\"23\"><br id=\"24\"><br id=\"25\"><br id=\"26\"><br id=\"27\"><br id=\"28\"><br id=\"29\"><br id=\"30\"><br id=\"31\">"
        )
    }

    #[test]
    fn bool_attr_works() {
        let html = render(input().type_("checkbox").checked());

        assert_eq!(html, r#"<input type="checkbox" checked>"#)
    }

    #[test]
    fn multiple_attrs_spaced_correctly() {
        let html = render(input().type_("checkbox").checked().aria_label("label"));

        assert_eq!(
            html,
            r#"<input type="checkbox" checked aria-label="label">"#
        )
    }

    #[test]
    fn readme1_works() {
        let element = input()
            .attr("hx-post", "/")
            .attr("hx-target", ".target")
            .attr("hx-swap", "outerHTML")
            .attr("hx-push-url", "false");
        let html = render(element);

        assert_eq!(
            html,
            r#"<input hx-post="/" hx-target=".target" hx-swap="outerHTML" hx-push-url="false">"#
        )
    }

    #[test]
    fn readme2_works() {
        fn turbo_frame(children: Element) -> Element {
            element("turbo-frame", children)
        }
        let html = render(turbo_frame(div("inside turbo frame")).id("id"));

        assert_eq!(
            "<turbo-frame id=\"id\"><div>inside turbo frame</div></turbo-frame>",
            html
        );
    }

    #[test]
    fn styles_dedup() {
        let p1 = p("").css((
            "color-red background-green",
            vec![
                ".color-red{color:red;}".into(),
                ".background-green{background:green;}".into(),
            ],
        ));
        let p2 = p("").css((
            "color-red background-blue",
            vec![
                ".color-red{color:red;}".into(),
                ".background-blue{background:blue;}".into(),
            ],
        ));
        let div1 = div((p1, p2)).css(("color-red", vec![".color-red{color:red;}".into()]));
        let styles = styles(&div1);
        let mut styles = styles
            .split(".")
            .into_iter()
            .filter(|x| !x.is_empty())
            .map(|x| format!(".{}", x))
            .collect::<Vec<String>>();

        styles.sort();

        let mut expected: Vec<String> = vec![
            ".color-red{color:red;}".into(),
            ".background-blue{background:blue;}".into(),
            ".background-green{background:green;}".into(),
        ];

        expected.sort();

        assert_eq!(expected, styles);
    }
}
