use std::cell::RefCell;

use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
struct StdLib {
    io: IoLib,
    string: StringLib,
}

impl espyscript::Extern for StdLib {
    fn index<'host>(
        &'host self,
        index: espyscript::Value<'host>,
    ) -> Result<espyscript::Value<'host>, espyscript::interpreter::Error<'host>> {
        match index {
            espyscript::Value {
                storage: espyscript::Storage::String(index),
            } if &*index == "io" => Ok(espyscript::Storage::Borrow(&self.io).into()),
            espyscript::Value {
                storage: espyscript::Storage::String(index),
            } if &*index == "string" => Ok(espyscript::Storage::Borrow(&self.string).into()),
            index => Err(espyscript::Error::IndexNotFound {
                index,
                container: espyscript::Storage::Borrow(self).into(),
            }),
        }
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "std module")
    }
}

#[derive(Debug, Default)]
struct IoLib {
    print: IoPrintFn,
}

impl espyscript::Extern for IoLib {
    fn index<'host>(
        &'host self,
        index: espyscript::Value<'host>,
    ) -> Result<espyscript::Value<'host>, espyscript::interpreter::Error<'host>> {
        match index {
            espyscript::Value {
                storage: espyscript::Storage::String(index),
            } if &*index == "print" => Ok(espyscript::Storage::Borrow(&self.print).into()),
            index => Err(espyscript::Error::IndexNotFound {
                index,
                container: espyscript::Storage::Borrow(self).into(),
            }),
        }
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "std.io module")
    }
}

#[derive(Debug, Default)]
struct IoPrintFn {
    output: RefCell<String>,
}

impl espyscript::Extern for IoPrintFn {
    fn call<'host>(
        &'host self,
        argument: espyscript::Value<'host>,
    ) -> Result<espyscript::Value<'host>, espyscript::Error<'host>> {
        match argument {
            espyscript::Value {
                storage: espyscript::Storage::String(message),
            } => {
                let mut output = self.output.borrow_mut();
                output.push_str(&message);
                output.push('\n');
                Ok(espyscript::Storage::Unit.into())
            }
            argument => Err(espyscript::Error::TypeError {
                value: argument,
                ty: espyscript::Storage::StringType.into(),
            }),
        }
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "std.io.print function")
    }
}

#[derive(Debug, Default)]
struct StringLib {
    concat: StringConcatFn,
}

impl espyscript::Extern for StringLib {
    fn index<'host>(
        &'host self,
        index: espyscript::Value<'host>,
    ) -> Result<espyscript::Value<'host>, espyscript::interpreter::Error<'host>> {
        match index {
            espyscript::Value {
                storage: espyscript::Storage::String(index),
            } if &*index == "concat" => Ok(espyscript::Storage::Borrow(&self.concat).into()),
            index => Err(espyscript::Error::IndexNotFound {
                index,
                container: espyscript::Storage::Borrow(self).into(),
            }),
        }
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "std.string module")
    }
}

#[derive(Debug, Default)]
struct StringConcatFn;

impl espyscript::Extern for StringConcatFn {
    fn call<'host>(
        &'host self,
        argument: espyscript::Value<'host>,
    ) -> Result<espyscript::Value<'host>, espyscript::Error<'host>> {
        argument
            .into_tuple()?
            .values()
            .map(|value| match value {
                espyscript::Value {
                    storage: espyscript::Storage::String(s),
                } => Ok(s as &str),
                value => Err(espyscript::Error::TypeError {
                    value: value.clone(),
                    ty: espyscript::Storage::StringType.into(),
                }),
            })
            .collect::<Result<String, _>>()
            .map(|s| espyscript::Storage::String(s.into()).into())
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "std.string.concat function")
    }
}

#[wasm_bindgen]
pub fn espyscript_eval(src: &str) -> String {
    match espyscript::Program::try_from(src) {
        Ok(program) => match program.eval() {
            Ok(result) => match espyscript::Function::try_from(result) {
                Ok(function) => {
                    let std = StdLib::default();

                    match function
                        .piped(espyscript::Storage::Borrow(&std).into())
                        .eval()
                    {
                        Ok(result) => {
                            let result = format!("{result:#?}");
                            let output = std.io.print.output.into_inner();

                            format!(
                                "<pre id=\"console-output\">{output}</pre><pre id=\"return-value\">{result}</pre>"
                            )
                        }
                        Err(e) => {
                            let e = format!("{e:#?}");
                            let output = std.io.print.output.into_inner();
                            format!(
                                "<pre id=\"console-output\">{output}</pre><pre id=\"eval-error\">Failed to evaluate program: {e}</pre>"
                            )
                        }
                    }
                }
                Err(espyscript::Error::ExpectedFunction(value)) => {
                    format!("<pre id=\"return-value\">{value:?}</pre>")
                }
                Err(_) => unreachable!(),
            },
            Err(e) => {
                format!("<pre id=\"eval-error\">Failed to evaluate program: {e:?}</pre>")
            }
        },
        Err(e) => {
            format!("<pre id=\"parse-error\">Failed to parse program: {e:?}</pre>")
        }
    }
}
