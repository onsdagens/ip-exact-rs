use anyhow::Result;
use clap::Parser;
use ip_xact_rs::*;
use log::info;
use roxmltree::Node;
use std::{collections::HashMap, fs};
use walkdir::WalkDir;
const IP_XACT_DIR: &'static str = "pulpinoexperiment/ip-xact";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = IP_XACT_DIR)]
    path: String,
}

#[derive(Debug, Clone)]
struct RegisterBlock {
    address: String,
    range: String,
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
    let mut register_blocks: HashMap<String, RegisterBlock> = HashMap::new();
    for entry in files.clone() {
        let txt = fs::read_to_string(entry.path()).expect("Unable to read file");
        let doc = roxmltree::Document::parse(&txt)?;
        for n in doc.descendants() {
            if n.has_tag_name("addressSpace") {
                let segments_nodes = find_children_by_tag_name(n, "segments");
                let mut segment_nodes: Vec<Vec<Node>> = vec![];
                for node in segments_nodes {
                    segment_nodes.push(
                        node.children()
                            .into_iter()
                            .filter(|child| child.has_tag_name("segment"))
                            .collect(),
                    );
                }
                let segment_nodes: Vec<Node> = segment_nodes.concat();
                for block in segment_nodes {
                    let name = get_name(block).unwrap();
                    let offset = find_child_by_tag_name(block, "addressOffset")
                        .unwrap()
                        .text_storage()
                        .unwrap()
                        .to_string();
                    let range = find_child_by_tag_name(block, "range")
                        .unwrap()
                        .text_storage()
                        .unwrap()
                        .to_string();
                    register_blocks.insert(
                        name.to_ascii_lowercase(), //just lowercase it to ignore case for now
                        RegisterBlock {
                            address: offset,
                            range,
                            registers: HashMap::new(),
                        },
                    );
                }
            }
        }
    }
    for entry in files {
        let txt = fs::read_to_string(entry.path()).expect("Unable to read file");
        let doc = roxmltree::Document::parse(&txt)?;
        for n in doc.descendants() {
            if n.has_tag_name("memoryMap") {
                let map_name = get_name(n).unwrap();
                let blocks = find_children_by_tag_name(n, "addressBlock");
                let mut registers: Vec<Vec<Node>> = vec![];
                for block in blocks {
                    registers.push(find_children_by_tag_name(block, "register"))
                }
                let registers = registers.concat();
                let mut register_map: HashMap<String, Register> = HashMap::new();
                registers.into_iter().for_each(|reg| {
                    let name = get_name(reg).unwrap();
                    let offset = find_child_by_tag_name(reg, "addressOffset")
                        .unwrap()
                        .text_storage()
                        .unwrap()
                        .to_string();
                    let width = find_child_by_tag_name(reg, "size")
                        .unwrap()
                        .text_storage()
                        .unwrap()
                        .to_string();
                    let field_nodes = find_children_by_tag_name(reg, "field");
                    let mut fields: HashMap<String, Field> = HashMap::new();
                    field_nodes.into_iter().for_each(|node| {
                        let name = get_name(node).unwrap();
                        let bit_offset = find_child_by_tag_name(node, "bitOffset")
                            .unwrap()
                            .text_storage()
                            .unwrap()
                            .to_string();
                        let bit_width = find_child_by_tag_name(node, "bitWidth")
                            .unwrap()
                            .text_storage()
                            .unwrap()
                            .to_string();

                        fields.insert(
                            name,
                            Field {
                                bit_offset,
                                bit_width,
                            },
                        );
                    });
                    register_map.insert(
                        name.clone(),
                        Register {
                            offset,
                            width,
                            fields,
                        },
                    );
                });
                let mut block = register_blocks
                    .get(&map_name.to_ascii_lowercase())
                    .unwrap()
                    .clone();
                block.registers = register_map;

                // prune empty register blocks (typically memories)
                if block.registers.len() != 0 {
                    register_blocks.insert(map_name.to_ascii_lowercase(), block);
                } else {
                    register_blocks.remove(&map_name.to_ascii_lowercase());
                }
            }
        }
    }
    register_blocks.into_iter().for_each(|block| {
        println!(
            "Block {}, Address {}, Range {}",
            block.0, block.1.address, block.1.range
        );
        block.1.registers.into_iter().for_each(|reg| {
            println!(
                "  Register {}, Offset {}, Range {}",
                reg.0, reg.1.offset, reg.1.width
            );
            reg.1.fields.into_iter().for_each(|field| {
                println!(
                    "      Field {}, Bit Offset {}, Bit Width {}",
                    field.0, field.1.bit_offset, field.1.bit_width
                )
            })
        })
    });
    Ok(())
}
