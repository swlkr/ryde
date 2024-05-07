#![allow(non_snake_case)]

use ryde::*;

routes!(("/", get(get_slash)), ("/*files", get(get_files)));

embed_static_files!("examples/html/static");

#[main]
async fn main() {
    serve("::1:3000", routes()).await
}

async fn get_slash() -> Html {
    html! {
        <View>
            <Heading route=url!(get_slash)/>
            <P>"ryde with rust 🐎!"</P>
        </View>
    }
}

fn Heading(route: String) -> Component {
    html! {
        <h1 class="text-2xl text-gray-950 dark:text-amber-300 dark:bg-gray-950">
            you are here {&route}
        </h1>
    }
}

fn P(elements: Elements) -> Component {
    html! { <p class="text-base bg-gray-950 bg-amber-500 dark:bg-gray-950">{elements}</p> }
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html>
        <html>
            <head>{render_static_files!()}</head>
            <body>{elements}</body>
        </html>
    }
}
