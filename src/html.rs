#![allow(non_snake_case)]

pub use ryde_macros::html;
use std::borrow::Cow;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = html! {
            <!DOCTYPE html>
            <html lang="en">
                <head></head>
                <body>shtml</body>
            </html>
        }
        .to_string();

        assert_eq!(
            result,
            r#"<!DOCTYPE html><html lang="en"><head></head><body>shtml</body></html>"#
        );
    }

    #[test]
    fn it_works_with_blocks() {
        let x = 1;
        let result = html! { <div>{x}</div> }.to_string();

        assert_eq!(result, r#"<div>1</div>"#);
    }

    #[test]
    fn it_works_with_attr_blocks() {
        let class = "flex items-center h-full";
        let result = html! { <div class=class></div> }.to_string();

        assert_eq!(result, r#"<div class="flex items-center h-full"></div>"#);
    }

    #[test]
    fn it_works_with_components() {
        fn Hello(name: &str) -> Component {
            html! { <div>{name}</div> }
        }

        let x = "<script>shtml</script>";
        let result = html! { <Hello name=x/> }.to_string();

        assert_eq!(result, r#"<div>&lt;script&gt;shtml&lt;/script&gt;</div>"#);
    }

    #[test]
    fn it_works_with_attrs() {
        fn Hypermedia(target: &str) -> Component {
            html! { <div x-target=target></div> }
        }

        let x = "body";
        let result = html! { <Hypermedia target=x/> }.to_string();

        assert_eq!(result, r#"<div x-target="body"></div>"#);
    }

    #[test]
    fn it_works_with_escaped_components() {
        fn Hello(elements: Elements) -> Component {
            html! { {elements} }
        }

        let x = "<script>alert(\"owned\")</script>";
        let result = html! {
            <Hello>
                <div>{x}</div>
            </Hello>
        }
        .to_string();

        assert_eq!(
            result,
            r#"<div>&lt;script&gt;alert(&quot;owned&quot;)&lt;/script&gt;</div>"#
        );
    }

    #[test]
    fn it_works_with_components_with_attrs_and_children() {
        fn Heading(class: &str, els: Elements) -> Component {
            html! { <h1 class=class>{els}</h1> }
        }

        let result = html! {
            <Heading class="text-7xl text-red-500">
                <p>How now brown cow</p>
            </Heading>
        };

        assert_eq!(
            result.to_string(),
            r#"<h1 class="text-7xl text-red-500"><p>How now brown cow</p></h1>"#
        );
    }

    #[test]
    fn it_works_with_components_with_children() {
        fn Hello(name: &str, elements: Elements) -> Component {
            html! {
                {elements}
                <div>{name}</div>
            }
        }

        let x = "shtml";
        let result = html! {
            <Hello name=x>
                <span>"mr."</span>
            </Hello>
        }
        .to_string();

        assert_eq!(result, r#"<span>mr.</span><div>shtml</div>"#);
    }

    #[test]
    fn it_works_for_tables() {
        const SIZE: usize = 2;
        let mut rows = Vec::with_capacity(SIZE);
        for _ in 0..SIZE {
            let mut inner = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                inner.push(i);
            }
            rows.push(inner);
        }

        let component = html! {
            <table>
                {rows
                    .iter()
                    .map(|cols| {
                        html! {
                            <tr>
                                {cols
                                    .iter()
                                    .map(|col| html! { <td>{col}</td> })
                                    .collect::<Vec<_>>()}
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </table>
        };

        assert_eq!(
            component.to_string(),
            "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
        );
    }

    #[test]
    fn it_works_for_tables_with_components() {
        const SIZE: usize = 2;
        let mut rows = Vec::with_capacity(SIZE);
        for _ in 0..SIZE {
            let mut inner = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                inner.push(i);
            }
            rows.push(inner);
        }

        fn Table(rows: Elements) -> Component {
            html! { <table>{rows}</table> }
        }

        fn Row(cols: Elements) -> Component {
            html! { <tr>{cols}</tr> }
        }

        fn Col(i: Elements) -> Component {
            html! { <td>{i}</td> }
        }

        let component = html! {
            <Table>
                {rows
                    .iter()
                    .map(|cols| {
                        html! {
                            <Row>
                                {cols.iter().map(|i| html! { <Col>{i}</Col> }).collect::<Vec<_>>()}
                            </Row>
                        }
                    })
                    .collect::<Vec<_>>()}
            </Table>
        };

        assert_eq!(
            component.to_string(),
            "<table><tr><td>0</td><td>1</td></tr><tr><td>0</td><td>1</td></tr></table>"
        );
    }

    #[test]
    fn it_works_with_multiple_children_components() {
        fn Html(component: Elements) -> Component {
            html! {
                <!DOCTYPE html>
                <html lang="en">{component}</html>
            }
        }

        fn Head(component: Elements) -> Component {
            html! { <head>{component}</head> }
        }

        fn Body(component: Elements) -> Component {
            html! { <body>{component}</body> }
        }

        let component = html! {
            <Html>
                <Head>
                    <meta name="" description=""/>
                    <title>head</title>
                </Head>
                <Body>
                    <div>shtml</div>
                </Body>
            </Html>
        };

        assert_eq!(component.to_string(), "<!DOCTYPE html><html lang=\"en\"><head><meta name=\"\" description=\"\"/><title>head</title></head><body><div>shtml</div></body></html>");
    }

    #[test]
    fn it_works_with_fragments() {
        fn HStack(elements: Elements) -> Component {
            html! { <div class="flex gap-4">{elements}</div> }
        }

        let component = html! {
            <HStack>
                <>
                    <div>1</div>
                    <div>2</div>
                    <div>3</div>
                </>
            </HStack>
        };

        assert_eq!(
            component.to_string(),
            r#"<div class="flex gap-4"><div>1</div><div>2</div><div>3</div></div>"#
        );
    }

    #[test]
    fn it_works_with_simple_loops() {
        fn List(elements: Elements) -> Component {
            html! { <ul>{elements}</ul> }
        }

        fn Item(elements: Elements) -> Component {
            html! { <li>{elements}</li> }
        }

        let items = vec![1, 2, 3];

        let component = html! { <List>{items.iter().map(|i| html! { <Item>{i}</Item> }).collect::<Vec<_>>()}</List> };

        assert_eq!(
            component.to_string(),
            r#"<ul><li>1</li><li>2</li><li>3</li></ul>"#
        );
    }

    #[test]
    fn it_works_with_fragments_and_components() {
        fn HStack(elements: Elements) -> Component {
            html! { <div class="flex gap-4">{elements}</div> }
        }

        fn VStack(elements: Elements) -> Component {
            html! { <div class="flex flex-col gap-4">{elements}</div> }
        }

        let component = html! {
            <HStack>
                <VStack>
                    <div>1</div>
                    <div>2</div>
                </VStack>
            </HStack>
        };

        assert_eq!(
            component.to_string(),
            r#"<div class="flex gap-4"><div class="flex flex-col gap-4"><div>1</div><div>2</div></div></div>"#
        );
    }
}

pub type Elements = Component;

#[derive(Debug, PartialEq, Eq)]
pub struct Component {
    pub html: String,
}

pub trait Render {
    fn render_to_string(&self, buffer: &mut String);
}

macro_rules! impl_render_int {
    ($t:ty) => {
        impl Render for $t {
            fn render_to_string(&self, buffer: &mut String) {
                let mut b = itoa::Buffer::new();
                buffer.push_str(b.format(*self));
            }
        }
    };
}

macro_rules! impl_render_float {
    ($t:ty) => {
        impl Render for $t {
            fn render_to_string(&self, buffer: &mut String) {
                let mut b = ryu::Buffer::new();
                buffer.push_str(b.format(*self));
            }
        }
    };
}

impl_render_int!(u8);
impl_render_int!(i8);
impl_render_int!(u16);
impl_render_int!(i16);
impl_render_int!(i64);
impl_render_int!(u64);
impl_render_int!(i32);
impl_render_int!(u32);
impl_render_int!(usize);
impl_render_int!(isize);

impl_render_float!(f64);
impl_render_float!(f32);

impl Render for Component {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&self.html);
    }
}

impl Render for String {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(self))
    }
}

impl Render for &String {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(*self))
    }
}

impl Render for &str {
    fn render_to_string(&self, buffer: &mut String) {
        buffer.push_str(&escape(*self))
    }
}

impl<T> Render for Vec<T>
where
    T: Render,
{
    fn render_to_string(&self, buffer: &mut String) {
        self.iter().for_each(|s| s.render_to_string(buffer));
    }
}

impl<T> Render for Option<T>
where
    T: Render,
{
    fn render_to_string(&self, buffer: &mut String) {
        match self {
            Some(t) => t.render_to_string(buffer),
            None => {}
        }
    }
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.html))
    }
}

pub fn escape<'a, S: Into<Cow<'a, str>>>(input: S) -> Cow<'a, str> {
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
