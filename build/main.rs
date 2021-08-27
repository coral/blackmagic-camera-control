use std::env;
use std::path::Path;

mod gen;
mod protocol;

fn main() {
    let prt = protocol::BlackmagicCameraProtocol::new().unwrap();
    let mut cg = gen::Datagen::new(prt);

    //Command
    {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("command.rs");

        let cmd_file = cg.gen_command();
        std::fs::write(dest_path, cmd_file.as_bytes()).unwrap();
    }
}
