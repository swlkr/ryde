# html

html offers an ergonomic way to render html from plain rust functions

```sh
cargo add html
```

# Write some html

```rust
use html::*;

fn render(element: Element) -> String {
  html::render((
    doctype(),
    html((
      head((title("title"), meta().charset("utf-8"))),
      body(element)
    ))
  ))
}

#[cfg(test)]
mod tests {

  #[test]
  fn it_works() {
      assert_eq!(
        render(div("html")),
        "<!DOCTYPE html><html><head><title>title</title></head><body><div>html</div></body></html>"
      )
  }
}
```

# Custom attributes

```rust
use html::*;

fn htmx_input() -> Element {
  input()
    .attr("hx-post", "/")
    .attr("hx-target", ".target")
    .attr("hx-swap", "outerHTML")
    .attr("hx-push-url", "false")
}

fn main() {
  let html: String = render(htmx_input());
  // html == <input hx-post="/" hx-target=".target" hx-swap="outerHTML" hx-push-url="false">
}
```

# Custom elements

```rust
use html::*;

fn turbo_frame(children: Element) -> Element {
    element("turbo-frame", children)
}

fn main() {
  let html: String = render(turbo_frame(div("inside turbo frame")).id("id"));
  // <turbo-frame id="id">
  //   <div>inside turbo frame</div>
  // </turbo-frame>
}
```
