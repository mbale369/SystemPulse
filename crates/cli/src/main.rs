//! Présentation terminal sans GUI, réseau ni modification système.
use std::{env, process::ExitCode, thread};
use systempulse_collector::{Collector, Profile};

struct Options {
    count: u16,
    profile: Profile,
    processes: bool,
}
fn options(args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut options = Options {
        count: 1,
        profile: Profile::Balanced,
        processes: false,
    };
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--count" => {
                options.count = args
                    .next()
                    .ok_or("--count nécessite un nombre")?
                    .parse()
                    .map_err(|_| "--count nécessite un entier")?;
                if !(1..=1200).contains(&options.count) {
                    return Err("--count doit être compris entre 1 et 1200".into());
                }
            }
            "--profile" => {
                options.profile =
                    Profile::parse(&args.next().ok_or("--profile nécessite une valeur")?)?
            }
            "--processes" => options.processes = true,
            "snapshot" => {}
            _ => return Err("Argument inconnu ; utiliser --help".into()),
        }
    }
    Ok(Some(options))
}
fn run() -> Result<(), String> {
    let Some(options) = options(env::args().skip(1))? else {
        println!(
            "SystemPulse — lecture seule\nUsage : systempulse [snapshot] [--count 1..1200] [--profile eco|balanced|detailed] [--processes]\nSortie : un instantané JSON par ligne. Aucun argument de processus collecté.\nÉchantillonnage : eco 5s, balanced 3s, detailed 2s ; arrêt par Ctrl+C."
        );
        return Ok(());
    };
    let mut collector = Collector::new(options.profile)?;
    // Deux relevés espacés sont nécessaires pour les compteurs CPU/réseau.
    collector.sample(options.processes)?;
    thread::sleep(options.profile.interval());
    for index in 0..options.count {
        let snapshot = collector.sample(options.processes)?;
        println!(
            "{}",
            serde_json::to_string(&snapshot).map_err(|_| "Sérialisation des mesures impossible")?
        );
        if index + 1 < options.count {
            thread::sleep(options.profile.interval());
        }
    }
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("SystemPulse : {message}");
            ExitCode::FAILURE
        }
    }
}
