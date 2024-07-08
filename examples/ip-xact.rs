use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use std::{collections::HashMap, fs};
use walkdir::WalkDir;
const FILE: &'static str = "models/riscv_core.1.0.xml";
const IP_XACT_DIR: &'static str = "pulpinoexperiment/ip-xact";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = IP_XACT_DIR)]
    path: String,
}

#[derive(Debug, Clone)]
struct RegisterSegment {
    range: String,
    offset: String,
    register_blocks: HashMap<String, RegisterBlock>,
}
#[derive(Debug, Clone)]
struct RegisterBlock {
    address: String,
    width: String,
    registers: HashMap<String, Register>,
}
#[derive(Debug, Clone)]
struct Register {
    offset: String,
    width: String,
    fields: HashMap<String, Field>,
}
#[derive(Debug, Clone)]
struct Field {
    bit_offset: String,
    bit_width: String,
}
fn main() -> Result<()> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        // Apply globally
        .apply()?;

    let args = Args::parse();
    let mut files = vec![];
    for entry in WalkDir::new(args.path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let name = entry.file_name().to_str().unwrap_or("");
        if name.contains(".xml") {
            files.push(entry);
        }
    }
    let mut segments: HashMap<String, RegisterSegment> = HashMap::new();
    for entry in files.clone() {
        let txt = fs::read_to_string(entry.path()).expect("Unable to read file");
        info!("parsing file: {:?}", entry.path());
        let doc = roxmltree::Document::parse(&txt)?;
        for n in doc.descendants() {
            if n.has_tag_name("addressSpace") {
                for c in n.children() {
                    if c.has_tag_name("segments") {
                        for segment in c.children() {
                            let mut name = "".to_string();
                            let mut offset = "".to_string();
                            let mut range = "".to_string();
                            for value in segment.children() {
                                if value.has_tag_name("name") {
                                    name = value.text_storage().unwrap().to_string();
                                    //println!("Name: {:?}", name);
                                }
                                if value.has_tag_name("addressOffset") {
                                    offset = value.text_storage().unwrap().to_string();
                                    //println!("Offset: {:?}", offset);
                                }
                                if value.has_tag_name("range") {
                                    range = value.text_storage().unwrap().to_string();
                                    //println!("Range: {:?}", range);
                                }
                            }
                            if name != "" {
                                segments.insert(
                                    name.to_lowercase(),
                                    RegisterSegment {
                                        range,
                                        offset,
                                        register_blocks: HashMap::new(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    for (name, val) in segments.clone().into_iter() {
        info!(
            "segment name: {}\n segment address: {}\n segment range: {}",
            name, val.offset, val.range
        );
    }
    for entry in files {
        let txt = fs::read_to_string(entry.path()).expect("Unable to read file");
        info!("parsing file: {:?}", entry.path());
        let doc = roxmltree::Document::parse(&txt)?;
        for n in doc.descendants() {
            if n.has_tag_name("memoryMap") {
                for c in n.children() {
                    if c.has_tag_name("name") {
                        let name = c.text_storage().unwrap().to_string().to_lowercase();
                        info!(
                            "found memory map for:{}",
                            c.text_storage().unwrap().to_string()
                        );
                        let segment = if segments.get(&name).is_some() {
                            info!("found segment {}", name);
                            segments.get(&name)
                        } else {
                            info!("couldn't find name {}, checking descendants...", name);
                            let mut segment = None;

                            for d in c.descendants() {
                                if d.has_tag_name("name") {
                                    info!(
                                        "descendant has name: {}",
                                        d.text_storage().unwrap().to_string()
                                    );
                                    if segments.get(&name).is_some() {
                                        segment = Some(segments.get(&name).unwrap());
                                    }
                                }
                            }
                            segment
                        };
                        if segment.is_some() {
                            info!("Found segment {}", name);
                        } else {
                            panic!("Could not find segment {}", name);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
