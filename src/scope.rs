use crate::collect_scope::collect_scope;
use crate::dsp_map::DspMap;
use crate::matcher::Matcher;
use crate::state::_state;
use crate::state_base::_StateBase;
use crate::transpile_to_js::transpile_script;

pub fn parse_scope(script: &mut String, scopes: &mut Vec<String>) {
    let matchers: [Matcher; 3] = [Matcher::Dom, Matcher::Cam, Matcher::Sin];
    let mut indexes = (0, 0, 0);

    for m in matchers {
        while let Some(pat) = collect_scope(script, &m, true) {
            let ind = pat.index();
            indexes.0 += ind;

            let end_i = match pat.ext() {
                &Some(a) => a,
                None => panic!("Unvalid code executed!"),
            };

            scopes.push(pat.mp().to_string());
            script.replace_range(ind..end_i, "// ");
        }
    }
}

pub fn scopify(
    script: &mut String,
    mut scopes: Vec<String>,
    config: &DspMap,
    base: &mut _StateBase,
) {
    let mut sid: usize = 0;

    while let Some(i) = script.find("// ") {
        let v = scopes
            .get_mut(sid)
            .unwrap_or_else(|| panic!("Invalid substring // "));
        let lang = config.expect("lang");
        let build = config.expect("build");

        transpile_script(lang, build, v);
        _state(v, base);

        script.replace_range(i..i + 3, v);

        sid += 1
    }
}
