use std::collections::{BTreeMap, VecDeque};

fn recur_run(path: &str, mut res: BTreeMap<String, Vec<(String, String)>>) -> BTreeMap<String, Vec<(String, String)>> {
    let mut q: VecDeque<String> = VecDeque::new();
    q.push_back(path.to_string());
    res.insert(path.to_string(), Vec::new());

    while !q.is_empty() {
        let entry_path = q.pop_front().unwrap();    
        let entries = std::fs::read_dir(&entry_path).unwrap();
        

        for subentry in entries{
            let subentry = subentry.unwrap();
            let subentry_path = subentry.path().to_str().unwrap().to_string();
            let subentry_name = subentry.file_name().to_str().unwrap().to_string();

            let ref_entry = res.get_mut(&entry_path).unwrap();
            ref_entry.push((subentry_name.clone(),subentry_path.clone()));

            if subentry.file_type().unwrap().is_dir() {
                q.push_back(subentry_path.clone());
                res.insert(subentry_path, Vec::new());
            }
        }
    }
    res
}
fn print_tree(start_point: &(String,String), res: &BTreeMap<String, Vec<(String, String)>>) -> String{
    
    match res.get(&start_point.1){
        Some(entries) =>{
            let mut ret = format!("[\"{}\", new Directory([\n", start_point.0);
            for ent in entries{
                ret += &print_tree(ent, res);
            }
            ret += "])],\n";
            return ret;
        },
        None => {
            return format!("[\"{}\", new File(new TextEncoder().encode(read(\"{}\")))],\n", start_point.0, start_point.1);
        }
    }
}
fn main() {

    let args: Vec<String> = std::env::args().collect();

    assert!(args.len() >= 2);
    if args[1] == "ruby"
    {
        let mut result_js = std::fs::read_to_string("base-wasi-rb.js").unwrap();
        let mut fds_str = String::from("let fds = [\nnew OpenFile(new File([])),\nConsoleStdout.lineBuffered((msg) => print(`${msg}`)),\nConsoleStdout.lineBuffered((msg) => print(`${msg}`)),\nnew PreopenDirectory(\"/\", [\n[\"testcase.rb\",\nnew File(new TextEncoder().encode(read(\"../testcase_volume/testcase.rb\")))]]),");
        let path = "../rb-wasi-sandbox/ruby-wasm32-wasi";
        let init_map = recur_run(path, BTreeMap::new());
        let res = print_tree(&(path.to_string(),path.to_string()), &init_map);
        let ret = res.replacen(path, "/ruby-wasm32-wasi", 1);
        let ret = ret.replace(path, "./ruby-wasm32-wasi");
        let ret = ret.replace("]),[\"/ruby-wasm32-wasi\", new Directory([", ",");
        fds_str += &ret;
        fds_str.pop();
        fds_str.pop();
        fds_str.push(';');
        result_js = result_js.replacen("let fds = [];", &fds_str,1);
        std::fs::write("./module-ready-wasi-rb.js", result_js).unwrap();
    }
    else if args[1] == "python"
    {
        assert!(args.len() == 3);

        let python_runner = std::process::Command::new(&args[2])
                                            .arg("--version").output().unwrap();
        let python_version_result =  String::from_utf8(python_runner.stdout).unwrap();
        let python_version_info: Vec<&str> = python_version_result.split_ascii_whitespace().collect();
        let python_version = python_version_info[1].to_string();
        let py_versions:Vec<&str> = python_version.split('.').collect();

        let mut result_js = std::fs::read_to_string("base-wasi-py.js").unwrap();
        let mut fds_str = String::from("let fds = [\nnew OpenFile(new File([])),\nConsoleStdout.lineBuffered((msg) => print(`${msg}`)),\nConsoleStdout.lineBuffered((msg) => print(`${msg}`)),\nnew PreopenDirectory(\"/\", [\n[\"testcase.py\",\nnew File(new TextEncoder().encode(read(\"../testcase_volume/testcase.py\")))]]),");

        let pymodule_path = "../py-wasi-sandbox/Modules";
        let init_map = recur_run(pymodule_path, BTreeMap::new());
        let res = print_tree(&(pymodule_path.to_string(),pymodule_path.to_string()), &init_map);
        let ret = res.replacen(pymodule_path, "/Modules", 1);
        let ret = ret.replace(pymodule_path, "./Modules");
        let mut ret = ret.replacen("[\"/Modules\", new Directory([", "new PreopenDirectory(\"Modules\", [", 1);
        ret.pop();
        ret.pop();
        ret.pop();
        ret.push_str(",\n");
    
        fds_str += &ret;
    
        let mut wasm_pylib_path = String::from("../py-wasi-sandbox");
        let pyver_libpath= "/lib.wasi-wasm32-".to_string() + py_versions[0] +"." + py_versions[1];
        wasm_pylib_path += &pyver_libpath;
        let init_map = recur_run(&wasm_pylib_path, BTreeMap::new());
        let res = print_tree(&(wasm_pylib_path.clone(),wasm_pylib_path.to_string()), &init_map);
        let ret = res.replacen(&wasm_pylib_path, &pyver_libpath, 1);
        let ret = ret.replace(&wasm_pylib_path, &(".".to_string() + &pyver_libpath));
        let mut ret = ret.replacen(&("[\"".to_string() + &pyver_libpath + "\", new Directory(["), &("new PreopenDirectory(\"".to_string() + &pyver_libpath[1..] + "\", ["), 1);
        ret.pop();
        ret.pop();
        ret.pop();
        ret.push_str(",\n");
        fds_str += &ret;
    
        let py_libpath = "../py-wasi-sandbox/Lib";
        let init_map = recur_run(py_libpath, BTreeMap::new());
        let pylib_obj_in_js = print_tree(&(py_libpath.to_string(),py_libpath.to_string()), &init_map);
        let ret = pylib_obj_in_js.replacen(py_libpath, "/Lib", 1);
        let ret = ret.replace(py_libpath, "./Lib");
        let mut ret = ret.replacen("[\"/Lib\", new Directory([", "new PreopenDirectory(\"/Lib\", [", 1);
        ret.pop();
        ret.pop();
        ret.pop();
        fds_str += &ret;
        fds_str += &"\n];";
    
        result_js = result_js.replacen("let fds = [];", &fds_str,1);
        result_js = result_js.replacen("let env = [];", &("let env = [\"PYTHONPATH=".to_string() + &pyver_libpath + "\"];"),1);
    
        std::fs::write("./module-ready-wasi-py.js", result_js).unwrap();
    }
}
