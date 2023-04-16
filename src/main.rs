mod at_html;
mod collect_gen;
mod compiler;
mod component;
mod gen_id;
mod new;
mod scope;
mod state;
mod state_base;
mod template;
mod std_err;
mod var_not_allowed;
mod v8_parse;
mod pass;
mod cpu_error;
mod std_out;
mod input;
mod dsp_parser;
mod get_prop;
mod merge_js;

use crate::state_base::_StateBase;
use crate::compiler::compile;
use crate::new::new;
use crate::pass::pass;
use crate::std_out::std_out;
use crate::dsp_parser::dsp_parser;
use crate::std_err::ErrType::OSError;
use crate::std_err::StdErr;
use std::env;
use std::process::Command;
use crate::merge_js::merge_js;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let state_base = _StateBase::new();

    if args.len() == 1 {
        let ne = "ntc new {project_name} - Create a new project";
        let build = "ntc build {project_name} - Build your project";
        let start = "ntc start {project_name} - Start and build your project\n";

        let inst = format!("{ne}\n{build}\n{start}");

        std_out(inst.as_str())
    } else {
        let map;

        match args[1].as_str() {
            "new" => new(args.get(2).expect("Project name not provided")),
            "build" => {
                map = dsp_parser("./config.dsp");

                compile(state_base, map.clone());

                match map.get("pre_build") {
                    Some(c) => {
                        let mut com = c.split(" ").collect::<Vec<&str>>();

                        com.retain(|x| *x != "");

                        if !com.is_empty() {
                            let a  = match Command::new(com[0])
                                .args(com[1..].to_vec())
                                .output() {
                                Ok(e) => e.stdout,
                                Err(e) => {
                                    let err = StdErr::new(OSError, &*e.to_string());
                                    err.exec();

                                    Vec::new()
                                }
                            };

                            std_out(&String::from_utf8_lossy(&a));
                        }
                    }
                    None => pass()
                }

                merge_js(map.clone());
            },
            &_ => {}
        }
    }

}
