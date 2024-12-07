use std::{collections::{BTreeSet, HashSet}, fmt::Display, fs::File, io::BufRead, path::PathBuf};
use anyhow::{anyhow, Context, Result};
use clap::{builder::ValueParser, value_parser, ArgMatches};
use strum_macros::EnumIter;

use crate::{ebi_framework::ebi_file_handler::EbiFileHandler, ebi_traits::{ebi_trait_event_log::EbiTraitEventLog, ebi_trait_finite_language::EbiTraitFiniteLanguage, ebi_trait_finite_stochastic_language::EbiTraitFiniteStochasticLanguage, ebi_trait_iterable_stochastic_language::EbiTraitIterableStochasticLanguage, ebi_trait_queriable_stochastic_language::EbiTraitQueriableStochasticLanguage, ebi_trait_semantics::EbiTraitSemantics, ebi_trait_stochastic_deterministic_semantics::EbiTraitStochasticDeterministicSemantics, ebi_trait_stochastic_semantics::EbiTraitStochasticSemantics}, ebi_validate, math::fraction::{Fraction, FractionNotParsedYet}, multiple_reader::MultipleReader, text::Joiner};

use super::{ebi_command::{EbiCommand, EBI_COMMANDS}, ebi_file_handler::EBI_FILE_HANDLERS, ebi_object::{EbiObject, EbiObjectType, EbiTraitObject}, ebi_trait::{EbiTrait, FromEbiTraitObject}, importable::Importable, prom_link::{JavaObjectHandler, JAVA_OBJECT_HANDLERS_FRACTION, JAVA_OBJECT_HANDLERS_STRING, JAVA_OBJECT_HANDLERS_USIZE}};

pub enum EbiInput {
    Trait(EbiTraitObject, &'static EbiFileHandler),
    Object(EbiObject, &'static EbiFileHandler),
    String(String),
    Usize(usize),
    FileHandler(EbiFileHandler),
    Fraction(Fraction),
}

impl EbiInput {
    pub fn to_type<T: FromEbiTraitObject + ?Sized>(self) -> Result<Box<T>> {
        FromEbiTraitObject::from_trait_object(self)
    }

    pub fn get_type(&self) -> EbiInputType {
        match self {
            EbiInput::Trait(t, _) => EbiInputType::Trait(t.get_trait()),
            EbiInput::Object(o, _) => EbiInputType::Object(o.get_type()),
            EbiInput::String(_) => EbiInputType::String,
            EbiInput::Usize(_) => EbiInputType::Usize,
            EbiInput::FileHandler(_) => EbiInputType::FileHandler,
            EbiInput::Fraction(_) => EbiInputType::Fraction,
        }
    }
}

#[derive(PartialEq,Eq,EnumIter)]
pub enum EbiInputType {
    Trait(EbiTrait),
    Object(EbiObjectType),
    AnyObject,
    FileHandler,
    String,
    Usize,
    Fraction,
}

impl EbiInputType {

    pub fn get_article(&self) -> &str {
        match self {
            EbiInputType::Trait(t) => t.get_article(),
            EbiInputType::Object(o) => o.get_article(),
            EbiInputType::AnyObject => "an",
            EbiInputType::String => "a",
            EbiInputType::Usize => "an",
            EbiInputType::FileHandler => "a",
            EbiInputType::Fraction => "a",
        }
    }

    pub fn get_parser_of_list(traits: &[&EbiInputType]) -> ValueParser {
        match traits[0] {
            EbiInputType::Trait(_) => value_parser!(PathBuf),
            EbiInputType::Object(_) => value_parser!(PathBuf),
            EbiInputType::AnyObject => value_parser!(PathBuf),
            EbiInputType::String => value_parser!(String).into(),
            EbiInputType::Usize => value_parser!(usize).into(),
            EbiInputType::FileHandler => value_parser!(EbiFileHandler).into(),
            EbiInputType::Fraction => value_parser!(FractionNotParsedYet).into(),
        }
    }

    pub fn get_java_object_handlers(&self) -> Vec<&JavaObjectHandler> {
        match self {
            EbiInputType::Trait(t) => {
                Self::get_file_handlers_java(t.get_file_handlers())
            },
            EbiInputType::Object(o) => {
                Self::get_file_handlers_java(o.get_file_handlers())
            },
            EbiInputType::AnyObject => {
                Self::get_file_handlers_java(EBI_FILE_HANDLERS.iter().collect())
            },
            EbiInputType::String => {
                let mut x = vec![];
                x.extend(JAVA_OBJECT_HANDLERS_STRING);
                x
            },
            EbiInputType::Usize => {
                let mut x = vec![];
                x.extend(JAVA_OBJECT_HANDLERS_USIZE);
                x
            },
            EbiInputType::FileHandler => {
                //not supported in Java;
                vec![]
            },
            EbiInputType::Fraction => {
                let mut x = vec![];
                x.extend(JAVA_OBJECT_HANDLERS_FRACTION);
                x
            },
        }
    }

    pub fn get_possible_inputs(traits: &[&'static EbiInputType]) -> Vec<String> {
        let mut result = HashSet::new();

        for input_type in traits {
            match input_type {
                EbiInputType::Trait(t) => {
                    result.extend(Self::show_file_handlers(t.get_file_handlers()));
                },
                EbiInputType::Object(o) => {
                    result.extend(Self::show_file_handlers(o.get_file_handlers()));
                },
                EbiInputType::AnyObject => {
                    result.extend(Self::show_file_handlers(EBI_FILE_HANDLERS.iter().collect()));
                },
                EbiInputType::String => {result.insert("text".to_string());},
                EbiInputType::Usize => {result.insert("integer".to_string());},
                EbiInputType::FileHandler => {
                    let extensions: Vec<String> = EBI_FILE_HANDLERS.iter().map(|file_type| file_type.file_extension.to_string()).collect();
                    result.insert("the file extension of any file type supported by Ebi (".to_owned() + &extensions.join_with(", ", " or ") + ")");
                },
                EbiInputType::Fraction => {result.insert("fraction".to_string());},
            };
        }

        result.into_iter().collect::<Vec<_>>()
    }

    pub fn get_possible_inputs_with_latex(traits: &[&'static EbiInputType]) -> Vec<String> {
        let mut result = HashSet::new();

        for input_type in traits {
            match input_type {
                EbiInputType::Trait(t) => {
                    result.extend(Self::show_file_handlers_latex(t.get_file_handlers()));
                },
                EbiInputType::Object(o) => {
                    result.extend(Self::show_file_handlers_latex(o.get_file_handlers()));
                },
                EbiInputType::AnyObject => {
                    result.extend(Self::show_file_handlers_latex(EBI_FILE_HANDLERS.iter().collect()));
                },
                EbiInputType::String => {result.insert("text".to_string());},
                EbiInputType::Usize => {result.insert("integer".to_string());},
                EbiInputType::FileHandler => {
                    let extensions: Vec<String> = EBI_FILE_HANDLERS.iter().map(|file_type| file_type.file_extension.to_string()).collect();
                    result.insert("the file extension of any file type supported by Ebi (".to_owned() + &extensions.join_with(", ", " or ") + ")");
                },
                EbiInputType::Fraction => {result.insert("fraction".to_string());},
            };
        }

        result.into_iter().collect::<Vec<_>>()
    }

    pub fn get_possible_inputs_with_java(traits: &[&'static EbiInputType]) -> Vec<JavaObjectHandler> {
        let mut result = HashSet::new();

        for input_type in traits {
            result.extend(input_type.get_java_object_handlers());
        }

        result = result.into_iter().filter(|java_object_handler| java_object_handler.translator_java_to_ebi.is_some()).collect();

        result.into_iter().cloned().collect::<Vec<_>>()
    }

    pub fn possible_inputs_as_strings_with_articles(traits: &[&'static EbiInputType], last_connector: &str) -> String {
        let list = Self::get_possible_inputs(traits);
        list.join_with(", ", last_connector)
    }

    pub fn show_file_handlers(file_handlers: Vec<&'static EbiFileHandler>) -> Vec<String> {
        file_handlers.iter().map(|file_handler| format!("{}", file_handler)).collect::<Vec<_>>()
    }

    pub fn show_file_handlers_latex(file_handlers: Vec<&'static EbiFileHandler>) -> Vec<String> {
        file_handlers.iter().map(|file_handler| format!("\\hyperref[filehandler:{}]{{{}}}", file_handler.name, file_handler)).collect::<Vec<_>>()
    }

    pub fn get_file_handlers_java(file_handlers: Vec<&'static EbiFileHandler>) -> Vec<&JavaObjectHandler> {
        file_handlers.iter().fold(vec![], |mut list, file_handler| {list.extend(file_handler.java_object_handlers); list})
    }
    
    pub fn get_applicable_commands(&self) -> BTreeSet<Vec<&'static EbiCommand>> {
        let mut result = EBI_COMMANDS.get_command_paths();
        result.retain(|path| {
            if let EbiCommand::Command { input_types, .. } = path[path.len() - 1] {
                for input_typess in input_types.iter() {
                    for input_typesss in input_typess.iter() {
                        if input_typesss == &self {
                            return true;
                        }
                    }
                }
            }
            false
        });
        result
    }
}

impl Display for EbiInputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EbiInputType::Trait(t) => t.fmt(f),
            EbiInputType::Object(o) => o.fmt(f),
            EbiInputType::AnyObject => write!(f, "object"),
            EbiInputType::String => write!(f, "text"),
            EbiInputType::Usize => write!(f, "integer"),
            EbiInputType::FileHandler => write!(f, "file"),
            EbiInputType::Fraction => write!(f, "fraction"),
        }
    }
}

#[derive(Debug)]
pub enum EbiTraitImporter {
    FiniteLanguage(fn(&mut dyn BufRead) -> Result<Box<dyn EbiTraitFiniteLanguage>>), //finite set of traces
    FiniteStochasticLanguage(fn(&mut dyn BufRead) -> Result<Box<dyn EbiTraitFiniteStochasticLanguage>>), //finite number of traces
    QueriableStochasticLanguage(fn(&mut dyn BufRead) -> Result<Box<dyn EbiTraitQueriableStochasticLanguage>>), //can query for the probability of a trace
    IterableStochasticLanguage(fn(&mut dyn BufRead) -> Result<Box<dyn EbiTraitIterableStochasticLanguage>>), //can walk over the traces, potentially forever
    EventLog(fn(&mut dyn BufRead) -> Result<Box<dyn EbiTraitEventLog>>), //full XES; access to traces and attributes
    Semantics(fn(&mut dyn BufRead) -> Result<EbiTraitSemantics>), //can walk over states  using transitions, potentially forever
    StochasticSemantics(fn(&mut dyn BufRead) -> Result<EbiTraitStochasticSemantics>), //can walk over states  using transitions, potentially forever
    StochasticDeterministicSemantics(fn(&mut dyn BufRead) -> Result<EbiTraitStochasticDeterministicSemantics>), //can walk over states using activities, potentially forever
}

impl EbiTraitImporter {
    pub fn get_trait(&self) -> EbiTrait {
        match self {
            EbiTraitImporter::FiniteLanguage(_) => EbiTrait::FiniteLanguage,
            EbiTraitImporter::FiniteStochasticLanguage(_) => EbiTrait::FiniteStochasticLanguage,
            EbiTraitImporter::QueriableStochasticLanguage(_) => EbiTrait::QueriableStochasticLanguage,
            EbiTraitImporter::IterableStochasticLanguage(_) => EbiTrait::IterableStochasticLanguage,
            EbiTraitImporter::EventLog(_) => EbiTrait::EventLog,
            EbiTraitImporter::Semantics(_) => EbiTrait::Semantics,
            EbiTraitImporter::StochasticSemantics(_) => EbiTrait::StochasticSemantics,
            EbiTraitImporter::StochasticDeterministicSemantics(_) => EbiTrait::StochasticDeterministicSemantics,
        }
    }

    pub fn import(&self, reader: &mut dyn BufRead) -> Result<EbiTraitObject> {
        Ok(match self {
            EbiTraitImporter::FiniteLanguage(f) => EbiTraitObject::FiniteLanguage((f)(reader)?),
            EbiTraitImporter::FiniteStochasticLanguage(f) => EbiTraitObject::FiniteStochasticLanguage((f)(reader)?),
            EbiTraitImporter::QueriableStochasticLanguage(f) => EbiTraitObject::QueriableStochasticLanguage((f)(reader)?),
            EbiTraitImporter::IterableStochasticLanguage(f) => EbiTraitObject::IterableStochasticLanguage((f)(reader)?),
            EbiTraitImporter::EventLog(f) => EbiTraitObject::EventLog((f)(reader)?),
            EbiTraitImporter::Semantics(f) => EbiTraitObject::Semantics((f)(reader)?),
            EbiTraitImporter::StochasticSemantics(f) => EbiTraitObject::StochasticSemantics((f)(reader)?),
            EbiTraitImporter::StochasticDeterministicSemantics(f) => EbiTraitObject::StochasticDeterministicSemantics((f)(reader)?),
        })
    }
}

impl Display for EbiTraitImporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_trait().to_string())
    }
}

#[derive(Debug)]
pub enum EbiObjectImporter {
    EventLog(fn(&mut dyn BufRead) -> Result<EbiObject>),
    DirectlyFollowsModel(fn(&mut dyn BufRead) -> Result<EbiObject>),
    FiniteLanguage(fn(&mut dyn BufRead) -> Result<EbiObject>),
    FiniteStochasticLanguage(fn(&mut dyn BufRead) -> Result<EbiObject>),
    LabelledPetriNet(fn(&mut dyn BufRead) -> Result<EbiObject>),
    StochasticDeterministicFiniteAutomaton(fn(&mut dyn BufRead) -> Result<EbiObject>),
    StochasticLabelledPetriNet(fn(&mut dyn BufRead) -> Result<EbiObject>),
    LanguageOfAlignments(fn(&mut dyn BufRead) -> Result<EbiObject>),
    DeterministicFiniteAutomaton(fn(&mut dyn BufRead) -> Result<EbiObject>),
    ProcessTree(fn(&mut dyn BufRead) -> Result<EbiObject>),
    Executions(fn(&mut dyn BufRead) -> Result<EbiObject>),
    StochasticLanguageOfAlignments(fn(&mut dyn BufRead) -> Result<EbiObject>),
}

impl EbiObjectImporter {
    pub fn get_type(&self) -> EbiObjectType {
        match self {
            EbiObjectImporter::EventLog(_) => EbiObjectType::EventLog,
            EbiObjectImporter::DirectlyFollowsModel(_) => EbiObjectType::DirectlyFollowsModel,
            EbiObjectImporter::FiniteLanguage(_) => EbiObjectType::FiniteLanguage,
            EbiObjectImporter::FiniteStochasticLanguage(_) => EbiObjectType::FiniteStochasticLanguage,
            EbiObjectImporter::LabelledPetriNet(_) => EbiObjectType::LabelledPetriNet,
            EbiObjectImporter::StochasticDeterministicFiniteAutomaton(_) => EbiObjectType::StochasticDeterministicFiniteAutomaton,
            EbiObjectImporter::StochasticLabelledPetriNet(_) => EbiObjectType::StochasticLabelledPetriNet,
            EbiObjectImporter::LanguageOfAlignments(_) => EbiObjectType::LanguageOfAlignments,
            EbiObjectImporter::StochasticLanguageOfAlignments(_) => EbiObjectType::StochasticLanguageOfAlignments,
            EbiObjectImporter::DeterministicFiniteAutomaton(_) => EbiObjectType::DeterministicFiniteAutomaton,
            EbiObjectImporter::ProcessTree(_) => EbiObjectType::ProcessTree,
            EbiObjectImporter::Executions(_) => EbiObjectType::Executions,
        }
    }
    
    pub fn get_importer(&self) -> fn(&mut dyn BufRead) -> Result<EbiObject> {
        match self {
            EbiObjectImporter::EventLog(importer) => *importer,
            EbiObjectImporter::DirectlyFollowsModel(importer) => *importer,
            EbiObjectImporter::FiniteLanguage(importer) => *importer,
            EbiObjectImporter::FiniteStochasticLanguage(importer) => *importer,
            EbiObjectImporter::LabelledPetriNet(importer) => *importer,
            EbiObjectImporter::StochasticDeterministicFiniteAutomaton(importer) => *importer,
            EbiObjectImporter::StochasticLabelledPetriNet(importer) => *importer,
            EbiObjectImporter::LanguageOfAlignments(importer) => *importer,
            EbiObjectImporter::StochasticLanguageOfAlignments(importer) => *importer,
            EbiObjectImporter::DeterministicFiniteAutomaton(importer) => *importer,
            EbiObjectImporter::ProcessTree(importer) => *importer,
            EbiObjectImporter::Executions(importer) => *importer,
        }
    }
}

impl Display for EbiObjectImporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_type().to_string())
    }
}

pub fn validate<X: Importable> (reader: &mut dyn BufRead) -> Result<()> {
    match X::import(reader) {
        Ok(_) => Ok(()),
        Err(x) => Err(x),
    }
}

pub fn get_reader_file(from_file: &PathBuf) -> Result<MultipleReader> {
    if from_file.as_os_str() == "-" {
        return MultipleReader::from_stdin();
    } else {
        let file = File::open(from_file).with_context(|| format!("Could not read file `{}`.", from_file.display()))?;
        return Ok(MultipleReader::from_file(file));
    }
}

pub fn get_reader(cli_matches: &ArgMatches, cli_id: &str) -> Result<MultipleReader> {
    if let Some(from_file) = cli_matches.try_get_one::<PathBuf>(cli_id)? {
        if from_file.as_os_str() == "-" {
            return MultipleReader::from_stdin();
        } else {
            let file = File::open(from_file).with_context(|| format!("Could not read file `{}`.", from_file.display()))?;
            return Ok(MultipleReader::from_file(file));
        }
    } else {
        return MultipleReader::from_stdin();
    }
}

pub fn read_as_trait(etrait: &EbiTrait, reader: &mut MultipleReader) -> Result<(EbiTraitObject, &'static EbiFileHandler)> {
    let mut error = None;
    for file_handler in EBI_FILE_HANDLERS {
        for importer in file_handler.trait_importers {
            if &importer.get_trait() == etrait {
                //attempt to import
                match importer.import(reader.get().context("Obtaining reader.")?.as_mut())
                    .with_context(|| {format!("The last attempted importer was: {}.", file_handler)})
                    .with_context(|| format!("Attempting to parse file as either {}.\nIf you know the type of your file, use `Ebi {}` to check it.", EbiInputType::show_file_handlers(etrait.get_file_handlers()).join(", "), ebi_validate!())) {
                    Ok(object) => {return Ok((object, file_handler));}, //object parsed, return it
                    Err(err) => {error = Some(err);}
                }
            }
        }
    }
    Err(error.unwrap())
}

pub fn read_as_object(etype: &EbiObjectType, reader: &mut MultipleReader) -> Result<(EbiObject, &'static EbiFileHandler)> {
    for file_handler in EBI_FILE_HANDLERS {
        for importer in file_handler.object_importers {
            if &importer.get_type() == etype {
                //attempt to import
                if let Ok(object) = (importer.get_importer())(reader.get().context("Could not obtain reader.")?.as_mut()) {
                    //object parsed; return it
                    return Ok((object, file_handler));
                }
            }
        }
    }
    Err(anyhow!("File could not be recognised."))
}

pub fn read_as_any_object(reader: &mut MultipleReader) -> Result<(EbiObject, &'static EbiFileHandler)> {
    for file_handler in EBI_FILE_HANDLERS {
        //attempt to import
        for importer in file_handler.object_importers {
            if let Ok(object) = (importer.get_importer())(reader.get().context("Could not obtain reader.")?.as_mut()) {
                //object parsed; return it
                return Ok((object, file_handler));
            }
        }
    }
    Err(anyhow!("File could not be recognised."))
}

pub fn validate_object_of(reader: &mut MultipleReader, file_handler: &EbiFileHandler) -> Result<()> {
    let result = (file_handler.validator)(reader.get()?.as_mut());
    return result;
}