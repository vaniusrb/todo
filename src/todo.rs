use crate::error_template::ErrorTemplate;
use cfg_if::cfg_if;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Todo {
    id: u16,
    title: String,
    completed: bool,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::{Connection, SqliteConnection};
        // use http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode};

        pub async fn db() -> Result<SqliteConnection, ServerFnError> {
            Ok(SqliteConnection::connect("sqlite:Todos.db").await?)
        }
    }
}

#[server(GetTodos, "/api")]
pub async fn get_todos(cx: Scope) -> Result<Vec<Todo>, ServerFnError> {
    // this is just an example of how to access server context injected in the handlers
    // http::Request doesn't implement Clone, so more work will be needed to do use_context() on this
    let req_parts = use_context::<leptos_axum::RequestParts>(cx);

    if let Some(req_parts) = req_parts {
        println!("Uri = {:?}", req_parts.uri);
    }

    use futures::TryStreamExt;

    let mut conn = db().await?;

    let mut todos = Vec::new();
    let mut rows = sqlx::query_as::<_, Todo>("SELECT * FROM todos").fetch(&mut conn);
    while let Some(row) = rows.try_next().await? {
        todos.push(row);
    }

    // Add a random header(because why not)
    // let mut res_headers = HeaderMap::new();
    // res_headers.insert(SET_COOKIE, HeaderValue::from_str("fizz=buzz").unwrap());

    // let res_parts = leptos_axum::ResponseParts {
    //     headers: res_headers,
    //     status: Some(StatusCode::IM_A_TEAPOT),
    // };

    // let res_options_outer = use_context::<leptos_axum::ResponseOptions>(cx);
    // if let Some(res_options) = res_options_outer {
    //     res_options.overwrite(res_parts).await;
    // }

    Ok(todos)
}

#[server(AddTodo, "/api")]
pub async fn add_todo(title: String) -> Result<(), ServerFnError> {
    let mut conn = db().await?;

    // fake API delay
    std::thread::sleep(std::time::Duration::from_millis(1250));

    match sqlx::query("INSERT INTO todos (title, completed) VALUES ($1, false)")
        .bind(title)
        .execute(&mut conn)
        .await
    {
        Ok(_row) => Ok(()),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[server(DeleteTodo, "/api")]
pub async fn delete_todo(id: u16) -> Result<(), ServerFnError> {
    let mut conn = db().await?;

    Ok(sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&mut conn)
        .await
        .map(|_| ())?)
}

#[component]
pub fn TodoApp(cx: Scope) -> impl IntoView {
    //let id = use_context::<String>(cx);
    provide_meta_context(cx);
    view! {
        cx,
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="leptos" href="/pkg/todo_app_sqlite_axum.css"/>
        <Router>
            <header>
                <h1>"My Tasks"</h1>
            </header>
            <main>
                <Routes>
                    <Route path="" view=Todos/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Todos(cx: Scope) -> impl IntoView {
    let add_todo = create_server_multi_action::<AddTodo>(cx);
    let delete_todo = create_server_action::<DeleteTodo>(cx);

    // list of todos is loaded from the server in reaction to changes
    let todos = create_local_resource(
        cx,
        move || (add_todo.version().get(), delete_todo.version().get()),
        move |_| get_todos(cx),
    );

    let existing_todos = {
        move || match todos.read(cx) {
            Some(Ok(todos)) => {
                if todos.is_empty() {
                    view! { cx, <p>"No tasks were found."</p> }.into_view(cx)
                } else {
                    todos
                        .into_iter()
                        .map(|todo| {
                            view! {
                                cx,
                                <tr>
                                    <td>{todo.title}</td>
                                    <td>
                                        <ActionForm action=delete_todo>
                                            <input type="hidden" name="id" value={todo.id}/>
                                            <input type="submit" value="X"/>
                                        </ActionForm>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view(cx)
                }
            }
            Some(Err(e)) => {
                view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
            }
            None => view! { cx, <pre>"Loading..."</pre> }.into_view(cx),
        }
    };

    view! {
        cx,
        <div>
            <MultiActionForm action=add_todo>
                <label>"Add a Todo"</label>
                <input type="text" class="input input-bordered w-full max-w-xs" name="title"/>
                <input type="submit" class="btn btn-success" value="Add"/>
            </MultiActionForm>
            "Existing todos:"
            <div class="overflow-x-auto">
                <table class="table">
                    {existing_todos}
                </table>
            </div>
        </div>
    }
}