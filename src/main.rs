use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

use fastwebsockets::upgrade;
use fastwebsockets::upgrade::UpgradeFut;
use fastwebsockets::Frame;
use fastwebsockets::OpCode;
use fastwebsockets::WebSocketError;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use tokio::net::TcpListener;

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::sync::Mutex;
use wry::WebViewBuilder;

fn websocket() {
    std::thread::spawn(move || {
        let mut current_value = Arc::new(Mutex::new(1));

        // async fn handle_client(fut: upgrade::UpgradeFut) -> Result<(), WebSocketError> {
        //     let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);

        //     loop {
        //         let frame = ws.read_frame().await?;
        //         match frame.opcode {
        //             OpCode::Close => break,
        //             OpCode::Text | OpCode::Binary => {
        //                 ws.write_frame(Frame::text(fastwebsockets::Payload::Borrowed(b"hello")))
        //                     .await?;
        //             }
        //             _ => {}
        //         }
        //     }

        //     Ok(())
        // }

        // let handle_client(fut: upgrade::UpgradeFut) -> Result<(), WebSocketError> {
        //     let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);

        //     loop {
        //         let frame = ws.read_frame().await?;
        //         match frame.opcode {
        //             OpCode::Close => break,
        //             OpCode::Text | OpCode::Binary => {
        //                 ws.write_frame(Frame::text(fastwebsockets::Payload::Borrowed(b"hello")))
        //                     .await?;
        //             }
        //             _ => {}
        //         }
        //     }

        //     Ok(())
        // }

        async fn server_upgrade(
            mut req: Request<Incoming>,
        ) -> Result<Response<Empty<Bytes>>, WebSocketError> {
            let (response, fut) = upgrade::upgrade(&mut req)?;

            // let mut current_value_clone = current_value.borrow_mut();
            tokio::task::spawn(async move {
                let mut ws = fastwebsockets::FragmentCollector::new(fut.await.unwrap());

                loop {
                    let frame = ws.read_frame().await.unwrap();
                    match frame.opcode {
                        OpCode::Close => break,
                        OpCode::Text | OpCode::Binary => {
                            ws.write_frame(Frame::text(fastwebsockets::Payload::Borrowed(
                                b"hello",
                            )))
                            .await
                            .unwrap();
                        }
                        _ => {}
                    }
                }
            });

            Ok(response)
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        rt.block_on(async move {
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            println!("Server started, listening on {}", "127.0.0.1:8080");
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                println!("Client connected");
                let current_value_clone = current_value.clone();
                tokio::spawn(async move {
                    let io = hyper_util::rt::TokioIo::new(stream);
                    let conn_fut = http1::Builder::new()
                        .serve_connection(
                            io,
                            service_fn(|mut req| {
                                let current_value_clone = current_value_clone.clone();

                                async move {
                                    let (response, fut) = upgrade::upgrade(&mut req)?;
                                    //Result<Response<Empty<Bytes>>, WebSocketError>
                                    let current_value_clone = current_value_clone.clone();

                                    tokio::task::spawn(async move {
                                        let mut ws = fastwebsockets::FragmentCollector::new(
                                            fut.await.unwrap(),
                                        );
                                        let mut current_value = 0;

                                        loop {
                                            // let mut current_value =
                                            //     current_value_clone.lock().await;
                                            // let current_value_str = format!("{}", *current_value);
                                            // *current_value += 1;
                                            // println!("{}", current_value_str);

                                            // current_value += 1;
                                            let current_value_str =
                                                r#"{"field1":"aa11","field2":"aa22", "structField":{"sf1" : "aaa11","sf2" : "aaa22"}}"#;

                                            let frame = ws.read_frame().await.unwrap();
                                            match frame.opcode {
                                                OpCode::Close => break,
                                                OpCode::Text | OpCode::Binary => {
                                                    let text = std::str::from_utf8(&frame.payload).unwrap();
                                                    // ws.write_frame(Frame::text(
                                                    //     fastwebsockets::Payload::Borrowed(
                                                    //         current_value_str.as_bytes(),
                                                    //     ),
                                                    // ))
                                                    ws.write_frame(Frame::text(
                                                        fastwebsockets::Payload::Borrowed(
                                                            &text.to_uppercase().as_bytes(),
                                                        )
                                                    ))
                                                    .await
                                                    .unwrap();
                                                }
                                                _ => {}
                                            }
                                        }
                                    });

                                    Ok(response) as Result<Response<Empty<Bytes>>, WebSocketError>
                                }
                            }),
                        )
                        .with_upgrades();
                    if let Err(e) = conn_fut.await {
                        println!("An error occurred: {:?}", e);
                    }
                });
            }
        })
    });

    // Copyright 2020-2023 Tauri Programme within The Commons Conservancy
    // SPDX-License-Identifier: Apache-2.0
    // SPDX-License-Identifier: MIT

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let builder = WebViewBuilder::new(&window);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let builder = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
    };

    let _webview = builder
        // tell the webview to load the custom protocol
        .with_devtools(true)
        .with_html(
            r#"
<!DOCTYPE html>
<html>
    <head>
        <title>Wry</title>
    </head>
<body>
<button id="test">Test</button>
<textarea id="output"></textarea>
        <script>
            var ws = new WebSocket('ws://127.0.0.1:8080');
            const output = document.getElementById('output');
            ws.onopen = () => {
                // for (let i = 0; i < target; i++) {
                //     ws.send(i);
                //     // console.log(i);
                // }

                // setTimeout(() => {
                //     document.body.innerText = latest;
                // }, 1000);
            };
            ws.onmessage = (event) => {
                // const start = parseInt(event.data);
                // callbacks[start]();
                // callbacks[start] = null;

                // const end = new Date().getTime();
                // const elapsed = end - start;

                // console.log(elapsed, event.data);
                output.value = event.data;
            };
            ws.onclose = () => {
                console.log('Connection closed');
            };

            output.onkeydown = (e) => {
                e.preventDefault();
                ws.send(output.value + e.key);
            }

        </script>
</body>
</html>
        "#,
        )
        .build()
        .unwrap();

    _webview.open_devtools();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit
        }
    });
}

// fn custom_protocol() {
//     let mut current_value = RefCell::new(0);

//     // Copyright 2020-2023 Tauri Programme within The Commons Conservancy
//     // SPDX-License-Identifier: Apache-2.0
//     // SPDX-License-Identifier: MIT

//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();

//     #[cfg(any(
//         target_os = "windows",
//         target_os = "macos",
//         target_os = "ios",
//         target_os = "android"
//     ))]
//     let builder = WebViewBuilder::new(&window);

//     #[cfg(not(any(
//         target_os = "windows",
//         target_os = "macos",
//         target_os = "ios",
//         target_os = "android"
//     )))]
//     let builder = {
//         use tao::platform::unix::WindowExtUnix;
//         use wry::WebViewBuilderExtUnix;
//         let vbox = window.default_vbox().unwrap();
//         WebViewBuilder::new_gtk(vbox)
//     };

//     let current_value_copy = current_value.clone();

//     let _webview = builder
//         .with_custom_protocol("wry".into(), move |request| {
//             let mut current_value = current_value_copy.borrow_mut();
//             let value = serde_json::Value::Number((*current_value).into());
//             (*current_value) += 1;
//             let content = serde_json::to_string(&value).unwrap();
//             println!("Request: {:?} {}", request, current_value);

//             Response::builder()
//                 .header(CONTENT_TYPE, "application/json")
//                 .header("Access-Control-Allow-Origin", "*")
//                 .body(content.as_bytes().to_vec())
//                 .unwrap()
//                 .map(Into::into)
//         })
//         // tell the webview to load the custom protocol
//         .with_devtools(true)
//         .with_html(
//             r#"
// <!DOCTYPE html>
// <html>
//     <head>
//         <title>Wry</title>
//         <script>
//             async function query() {
//                 const response = await fetch('wry://');
//                 const json = await response.json();
//                 document.body.innerText = json;
//             }

//             document.onload = () => {
//                 query()
//             }
//         </script>
//     </head>
// <body>
// </body>
// </html>
//         "#,
//         )
//         .build()
//         .unwrap();

//     _webview.open_devtools();

//     event_loop.run(move |event, _, control_flow| {
//         *control_flow = ControlFlow::Wait;

//         if let Event::WindowEvent {
//             event: WindowEvent::CloseRequested,
//             ..
//         } = event
//         {
//             *control_flow = ControlFlow::Exit
//         }
//     });
// }

// fn get_wry_response(
//     request: Request<Vec<u8>>,
// ) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
//     let path = request.uri().path();
//     // Read the file content from file path
//     let root = PathBuf::from("examples/custom_protocol");
//     let path = if path == "/" {
//         "index.html"
//     } else {
//         //  removing leading slash
//         &path[1..]
//     };
//     let content = std::fs::read(std::fs::canonicalize(root.join(path))?)?;

//     // Return asset contents and mime types based on file extentions
//     // If you don't want to do this manually, there are some crates for you.
//     // Such as `infer` and `mime_guess`.
//     let mimetype = if path.ends_with(".html") || path == "/" {
//         "text/html"
//     } else if path.ends_with(".js") {
//         "text/javascript"
//     } else if path.ends_with(".png") {
//         "image/png"
//     } else if path.ends_with(".wasm") {
//         "application/wasm"
//     } else {
//         unimplemented!();
//     };

//     Response::builder()
//         .header(CONTENT_TYPE, mimetype)
//         .body(content)
//         .map_err(Into::into)
// }

fn main() {
    if std::env::args().any(|arg| arg == "custom_protocol") {
        // custom_protocol();
    } else if std::env::args().any(|arg| arg == "websocket") {
        websocket();
    } else {
        panic!("Please provide a valid bench name");
    }
}
