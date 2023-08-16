use crate::at_gen_id::_gen_id;
use crate::at_temp::at_temp;
use crate::collect_scope::collect_scope;
use crate::component::Component;
use crate::component::{component, stringify_component};
use crate::config::Config;
use crate::expected::expect_some;
use crate::import_base::ImportBase;
use crate::import_lib::import_lib;
use crate::import_npm::import_npm;
use crate::import_script::import_script;
use crate::js_module::module;
use crate::matcher::Matcher;
use crate::pass::pass;
use crate::scope::{parse_scope, scopify};
use crate::state::_state;
use crate::state_base::_StateBase;
use crate::std_err::{ErrType::OSError, StdErr};
use crate::sys_exec::sys_exec;
use crate::template::template;
use crate::consts::IGNORE_STATE;
use crate::udt::UDT;
use rusty_v8::{self as v8, json::stringify, Script};
use serde_json::{Map, Value};
use std::fs::{read_to_string, write};
use crate::comment::comment;

pub fn compile(mut state: _StateBase, mut import_base: ImportBase, config: &Config) {
    let binding = String::from("js");
    let ext = config.get("lang").unwrap_or(&binding);

    let binding = String::new();
    let command = config.get("build").unwrap_or(&binding);

    let src = &format!("./src/app.{ext}");

    let mut app = read_to_string(src).expect("Project or app.nts not found");

    app = app
        .lines()
        .map(|e| e.trim())
        .collect::<Vec<&str>>()
        .join("\n");

    comment(&mut app);

    let mut imports: Vec<Component> = vec![];
    let mut names: Vec<String> = vec![];

    let binding = String::from("app");
    let app_matcher = Matcher::Component(&binding);

    let pat = expect_some(collect_scope(&app, &app_matcher, false), "App component");

    let app_started = pat.index();
    let main_app = pat.mp_val();

    let split = main_app.split('\n');

    let mut js = String::new();
    let binding = Matcher::Template.to_string();
    let t = binding.as_str();

    for s in split {
        if s != t {
            js.push('\n');
            js.push_str(s)
        } else {
            break;
        }
    }

    let mut comp_html = expect_some(
        collect_scope(&main_app, &Matcher::Template, false),
        "Template",
    )
    .mp_val();

    let mut scopes: Vec<String> = Vec::new();

    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    while let Some(e) = app.find("import component") {
        let mut namei = e + 17;

        while &app[namei..namei + 4] != "from" {
            namei += 1;
        }

        let mut ci = namei + 5;

        while &app[ci..ci + 1] != "\n" {
            ci += 1
        }

        let cns = app[e + 16..namei].split(',');
        let fnm = &app[namei + 5..ci];

        let isolate = &mut v8::Isolate::new(Default::default());

        let scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        for cn in cns {
            names.push(cn.trim().to_string());
            imports.push(component(
                fnm.to_string(),
                cn.trim().to_string(),
                scope,
                &mut state,
                &mut import_base,
                command,
                config,
            ));
        }

        app.replace_range(e..ci + 1, "")
    }

    import_lib(&mut app, &mut import_base, &mut js, app_started);
    parse_scope(&mut js, &mut scopes);

    write(format!("./build/.$.{ext}"), js.clone()).unwrap_or_else(|e| panic!("{}", e));
    sys_exec(format!("{command} ./build/.$.{ext}"));

    js = read_to_string("./build/.$.js").unwrap_or(js.clone());

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    module(&mut app, &mut import_base, &mut js);
    import_script(&mut app, &mut import_base, &mut js);

    _gen_id(&mut js, &mut comp_html);

    let ben = &js.replace(".cam()", "");
    let code = v8::String::new(scope, ben).unwrap();

    let mut script = Script::compile(scope, code, None).unwrap();

    let _ = script.run(scope).unwrap();

    for n in names {
        let m = &format!("<{}", n);

        if let Some(e) = comp_html.find(m) {
            for i in &imports {
                if i.name == n {
                    let mut cde = e+n.len();

                    while &comp_html[cde..cde+2] != "/>" {
                        cde += 1;
                    }

                    comp_html.replace_range(e..cde+2, &i.html);

                    js.push('\n');
                    js.push_str(i.js.as_str())
                }
            }
        }
    }

    at_temp(&mut comp_html, &mut js,&mut state, scope);
    template(&mut comp_html, &mut js, scope, &mut state);
    _state(&mut js, &mut state);

    js = js.replace(IGNORE_STATE, "").
            replace(".cam()", "");

    match comp_html.find("<Router route=") {
        None => pass(),
        Some(a) => {
            if &comp_html[a + 14..a + 15] == "{" {
                let mut idx = a;

                while &comp_html[idx..idx + 1] != "}" {
                    idx += 1
                }

                let json = &comp_html[a + 14..idx + 1];

                match serde_json::from_str::<Value>(json) {
                    Ok(a) => {
                        let obj = a.as_object().unwrap();

                        let mut map = Map::new();

                        for (key, val) in obj {
                            let s = val.as_str().unwrap();
                            map.insert(
                                key.clone(),
                                Value::String(stringify_component(&component(
                                    s.to_string(),
                                    "Render".to_string(),
                                    scope,
                                    &mut state,
                                    &mut import_base,
                                    command,
                                    config,
                                ))),
                            );
                        }

                        js = format!(
                            "{js}\nvar Route = {}",
                            serde_json::to_string::<Value>(&Value::Object(map)).unwrap()
                        );
                    }
                    Err(_) => panic!("Can't even parse json in ohio"),
                }

                js = format!(
                    "{js}\n{}",
                    "\
    function main() {
        for (let k in Route) {
            if (window.location.pathname == k) {
                document.getElementById(\"app\").innerHTML = Route[k]
                window.history.pushState({}, \"\", k)
            }
        }
    }

    main()
                "
                );

                comp_html.replace_range(a..idx + 2, "")
            } else {
                let mut idx = a + 14;

                while &comp_html[idx..idx + 1] != "/" {
                    idx += 1
                }

                let name_ = comp_html[a + 14..idx].trim();

                let not_found = match config.get("404") {
                    Some(e) => stringify_component(&component(
                        e.clone(),
                        String::from("Page"),
                        scope,
                        &mut state,
                        &mut import_base,
                        command,
                        config
                    )),
                    None => "\
                        <pre style=\"word-wrap: break-word; white-space: pre-wrap;\">404 page not found</pre>
                    ".to_string()
                };

                write("./build/error.html", not_found.clone()).unwrap_or_else(|e| panic!("{e}"));

                let v8_str = v8::String::new(scope, name_).unwrap();

                script = Script::compile(scope, v8_str, None).unwrap();

                let res = script.run(scope).unwrap();

                let router = stringify(scope, res).unwrap().to_rust_string_lossy(scope);

                let binding: Value = serde_json::from_str::<Value>(&router).unwrap();

                let obj = binding.as_object().unwrap();

                let mut map = Map::new();
                map.insert(String::from("404"), Value::String(not_found));

                for (key, val) in obj {
                    let s = val.as_str().unwrap();
                    let _ = map.insert(
                        key.clone(),
                        Value::String(stringify_component(&component(
                            String::from(s),
                            String::from("Render"),
                            scope,
                            &mut state,
                            &mut import_base,
                            command,
                            config,
                        ))),
                    );
                }

                js = format!(
                    "{}\nvar Route = {}",
                    js,
                    serde_json::to_string::<Value>(&Value::Object(map)).unwrap()
                );

                js = format!(
                    "{js}\n{}",
                    "\
    function main() {
        var path = window.location.pathname
        let not_found = true
        for (let k in Route) {
            if (path == k) {
                not_found = false
                document.body.innerHTML = Route[k]
                window.history.pushState({}, \"\", k)
            }
        }
        if (not_found) {
            document.body.innerHTML = Route[\"404\"]
            window.history.pushState({}, \"\", path)
        }
    }
    main()
                "
                );

                comp_html.replace_range(a..idx + 2, "");
            }
        }
    }

    UDT(&mut comp_html, &mut js, &imports);

    let head = config.expect("head");

    import_npm(&mut app, &mut js);

    let binding = String::from("./build/dist.html");
    let _app_html = config.get("_app_html").unwrap_or(&binding);

    scopify(&mut js, scopes, config, &mut state);

    write(
        _app_html,
        format!(
            "
<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta name=\"description\" content=\"{}\">
    <meta name=\"keywords\" content=\"{}\">
    <meta name=\"author\" content=\"{}\">
    <title>{}</title>
    {head}
</head>
<body>
    {comp_html}
    <script>
        {js}
    </script>
<body>
</html>
",
            config.expect("description"),
            config.expect("keywords"),
            config.expect("author"),
            config.expect("title")
        ),
    )
    .unwrap_or_else(|e| StdErr::exec(OSError, &e.to_string()));
}
