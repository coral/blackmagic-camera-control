extern crate codegen;
use crate::protocol::{BlackmagicCameraProtocol, Parameter};
use codegen::{Block, Scope};
use convert_case::{Case, Casing};

static derive_traits: [&str; 4] = ["Debug", "PartialEq", "Clone", "PartialOrd"];

pub struct Datagen {
    protocol: BlackmagicCameraProtocol,
}

impl Datagen {
    pub fn new(protocol: BlackmagicCameraProtocol) -> Datagen {
        return Datagen { protocol };
    }

    pub fn gen_command(&mut self) -> String {
        let mut scope = Scope::new();

        Datagen::imports(&mut scope);
        Datagen::commands(&mut scope, &self.protocol);
        Datagen::parameters(&mut scope, &self.protocol);

        scope.to_string()
    }

    fn imports(s: &mut Scope) {
        s.import("fixed::types", "I5F11");
        s.import("thiserror", "Error");
        s.import(
            "crate::rawcommand",
            "{CommandError, RawCommand, ParamType, Parameter}",
        );
    }

    fn commands(s: &mut Scope, protocol: &BlackmagicCameraProtocol) {
        let mut data = s.new_enum("Command").vis("pub");
        for t in derive_traits {
            data.derive(t);
        }

        data.allow("dead_code");

        for category in protocol.groups.iter() {
            data.new_variant(&category.normalized_name.to_case(Case::UpperCamel))
                .tuple(&category.normalized_name.to_case(Case::UpperCamel));
        }

        let im = s.new_impl("Command");

        // from_raw func
        {
            let mut from_raw = Block::new("");
            from_raw.line("let raw_cmd = RawCommand::from_raw(data)?;");
            from_raw.line("match raw_cmd.category");

            let mut match_block = Block::new("");
            for category in protocol.groups.iter() {
                match_block.line(format!(
                    "{} => Ok(Command::{1}({1}::from_raw(raw_cmd)?)),",
                    &category.id,
                    &category.normalized_name.to_case(Case::UpperCamel)
                ));
            }
            match_block.line("_ => Err(CommandError::CategoryNotDefined)");
            from_raw.push_block(match_block);

            im.new_fn("from_raw")
                .arg("data", "&[u8]")
                .ret("Result<Self, CommandError>")
                .vis("pub")
                .push_block(from_raw);
        }
    }

    fn parameters(s: &mut Scope, protocol: &BlackmagicCameraProtocol) {
        //Enums
        for category in protocol.groups.iter() {
            let mut data = s
                .new_enum(&category.normalized_name.to_case(Case::UpperCamel))
                .vis("pub");
            for t in derive_traits {
                data.derive(t);
            }

            data.allow("dead_code");

            for param in category.parameters.iter() {
                let t = lookuptype(param);
                if t != "Void" {
                    data.new_variant(&param.normalized_parameter.to_case(Case::UpperCamel))
                        .tuple(t);
                } else {
                    data.new_variant(&param.normalized_parameter.to_case(Case::UpperCamel));
                }
            }
        }

        //Implementations of trait
        for category in protocol.groups.iter() {
            let im = s
                .new_impl(&category.normalized_name.to_case(Case::UpperCamel))
                .impl_trait("Parameter");

            //id function
            {
                let mut id_block = codegen::Block::new("");
                id_block.line("match self");

                let mut enum_block = codegen::Block::new("");
                for param in category.parameters.iter() {
                    if lookuptype(&param) == "Void" {
                        enum_block.line(format!(
                            "{}::{} => {},",
                            &category.normalized_name.to_case(Case::UpperCamel),
                            &param.normalized_parameter.to_case(Case::UpperCamel),
                            &param.id
                        ));
                    } else {
                        enum_block.line(format!(
                            "{}::{}(_) => {},",
                            &category.normalized_name.to_case(Case::UpperCamel),
                            &param.normalized_parameter.to_case(Case::UpperCamel),
                            &param.id
                        ));
                    }
                }
                id_block.push_block(enum_block);

                im.new_fn("id").arg_self().ret("u8").push_block(id_block);
            }

            //from_raw function
            {
                let mut raw_block = Block::new("");
                raw_block.line("match cmd.parameter");

                let mut match_block = Block::new("");

                for param in category.parameters.iter() {
                    if lookuptype(&param) != "Void" {
                        match_block.line(format!(
                            "{} => Ok({}::{}(ParamType::from_bytes(&cmd.data)?)),",
                            &param.id,
                            &category.normalized_name.to_case(Case::UpperCamel),
                            &param.normalized_parameter.to_case(Case::UpperCamel),
                        ));
                    } else {
                        match_block.line(format!(
                            "{} => Ok({}::{}),",
                            &param.id,
                            &category.normalized_name.to_case(Case::UpperCamel),
                            &param.normalized_parameter.to_case(Case::UpperCamel),
                        ));
                    }
                }

                match_block.line("_ => Err(CommandError::ParameterNotDefined),");

                raw_block.push_block(match_block);

                im.new_fn("from_raw")
                    .arg("cmd", "RawCommand")
                    .ret("Result<Self, CommandError>")
                    .push_block(raw_block);
            }
        }
    }
}

fn lookuptype(p: &Parameter) -> &'static str {
    if p.index.len() > 0 {
        match p.type_field.as_ref() {
            "void" => "Void",
            "int8" => "Vec<i8>",
            "int16" => "Vec<i16>",
            "int32" => "Vec<i32>",
            "int64" => "Vec<i64>",
            "string" => "String",
            "fixed16" => "Vec<f32>",
            _ => "Void",
        }
    } else {
        match p.type_field.as_ref() {
            "void" => "Void",
            "int8" => "i8",
            "int16" => "i16",
            "int32" => "i32",
            "int64" => "i64",
            "string" => "String",
            "fixed16" => "f32",
            _ => "Void",
        }
    }
}
