#![allow(non_snake_case)]

use ryde::*;

routes!(("/", get(get_slash)), ("/*files", get(get_files)));

embed_static_files!("examples/html/static");

#[main]
async fn main() {
    serve("::1:3000", routes()).await
}

async fn get_slash(uri: Uri) -> Html {
    html! {
        <View>
            <Heading route=uri.to_string()/>
            <Button>ryde with rust</Button>
        </View>
    }
}

fn Heading(route: String) -> Component {
    html! { <h1 class="text-2xl">"you are here " {&route}</h1> }
}

fn Button(elements: Elements) -> Component {
    html! {
        <button class="bg-orange-500 hover:bg-orange-400 text-white hover:dark:bg-orange-600 rounded-md px-4 py-2 text-center">
            {elements}
        </button>
    }
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html>
        <html>
            <head>{render_static_files!()}</head>
            <body class="grid place-content-center h-svh dark:bg-gray-950 gray-100 text-gray-950 dark:text-white">
                {elements}
            </body>
        </html>
    }
}
