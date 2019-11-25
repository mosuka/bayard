use std::time::Duration;

use clap::ArgMatches;
use crossbeam_channel::{select, tick};
use job_scheduler::{Job, JobScheduler};
use log::*;

use crate::client::client::{create_client, Clerk};
use crate::util::log::set_logger;
use crate::util::signal::sigterm_channel;

pub fn run_schedule_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

    let servers: Vec<_> = matches
        .values_of("SERVERS")
        .unwrap()
        .map(|addr| create_client(addr))
        .collect();
    let commit_schedule = matches.value_of("COMMIT_SCHEDULE").unwrap();
    let merge_schedule = matches.value_of("MERGE_SCHEDULE").unwrap();

    info!("start scheduler");

    let mut scheduler = JobScheduler::new();

    scheduler.add(Job::new(commit_schedule.parse().unwrap(), || {
        let client_id = rand::random();
        let mut client = Clerk::new(&servers, client_id);

        info!("commit");
        client.commit();
    }));

    scheduler.add(Job::new(merge_schedule.parse().unwrap(), || {
        let client_id = rand::random();
        let mut client = Clerk::new(&servers, client_id);

        info!("merge");
        client.merge();
    }));

    // Wait for signals for termination (SIGINT, SIGTERM).
    let ticks = tick(Duration::from_secs(1));
    let sigterm_receiver = sigterm_channel().unwrap();
    loop {
        select! {
            recv(ticks) -> _ => {
                scheduler.tick();
            }
            recv(sigterm_receiver) -> _ => {
                break;
            }
        }
    }

    info!("stop scheduler");

    Ok(())
}
