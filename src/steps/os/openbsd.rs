use std::fmt::{self, Display, Formatter};
use std::process::Command;
use crate::execution_context::ExecutionContext;
use crate::command::CommandExt;
use crate::terminal::{print_separator, print_info, print_warning};
use crate::executor::ExecutorOutput::Wet;
use crate::sudo::Sudo;
use color_eyre::eyre::Result;
use uname::Info;

#[derive(Debug, Clone, Copy)]
struct Version {
    major: u32,
    minor: u32,
}

impl Version {
    fn new(info: &Info) -> Self {
        let (major, minor) = info.release.split_once('.').expect("Failed to parse version.");
        Self {
            major: major.parse().expect("Failed to parse major version number."),
            minor: minor.parse().expect("Failed to parse minor version number."),
        }
    }

    fn next(self) -> Self {
        let mut major = self.major;
        let mut minor = self.minor;

        if minor < 9 {
            minor += 1;
        } else {
            major += 1;
            minor = 0;
        }

        Self { major, minor }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

fn with_sudo(ctx: &ExecutionContext, f: impl FnOnce(&Sudo) -> Result<()>) -> Result<()> {
    if let Some(sudo) = ctx.sudo() {
        f(sudo)
    } else {
        print_warning("No sudo detected. Skipping step.");
        Ok(())
    }
}

pub fn sysupgrade(ctx: &ExecutionContext) -> Result<()> {
    print_separator("OpenBSD System Update");

    let u = Info::new()?;
    let v = Version::new(&u).next();
    let installurl = std::fs::read_to_string("/etc/installurl")
        .unwrap_or_else(|_| "https://ftp.openbsd.org/pub/OpenBSD".to_string());
    let url = format!("{installurl}/{v}/{}/SHA256.sig", u.machine);

    // Check if an update exists.
    let out = Command::new("/usr/bin/ftp")
        .args(&["-V", "-o", "-", &url])
        .output_checked()?;

    if out.status.success() {
        print_info("New update available for OpenBSD {v}.");
        
        // Install update without rebooting (-n).
        with_sudo(ctx, |sudo| {
            ctx
                .run_type()
                .execute(sudo)
                .args(&["/usr/sbin/sysupgrade", "-n"])
                .status_checked()
        })?;

        print_warning("Please reboot to finish the update.");
    } else {
        print_info("No new updates found. Skipping.");
    }


    Ok(())
}

pub fn syspatch(ctx: &ExecutionContext) -> Result<()> {
    print_separator("OpenBSD Patches");

    with_sudo(ctx, |sudo| {
        print_info("Checking for patches...");

        // Check for available patches.
        let out = ctx
            .run_type()
            .execute(sudo)
            .args(&["/usr/sbin/syspatch", "-c"])
            .output()?;

        if let Wet(out) = out {
            if out.stdout.iter().any(|b| *b != b'\n') {
                print_info("New patches available:");

                // Print all available patches.
                out.stdout
                    .split(|b| *b == b'\n')
                    .filter(|line| !line.is_empty())
                    .try_for_each(|line| -> Result<()> {
                        let patch = std::str::from_utf8(line)?;
                        print_info(format!("- {patch}"));
                        Ok(())
                    })?;
                println!();

                // Install all available patches.
                ctx
                    .run_type()
                    .execute(sudo)
                    .args(&["/usr/sbin/syspatch"])
                    .status_checked()?;
            } else {
                print_info("No new available patches. Skipping.");
            }   
        }

        Ok(())
    })
}

pub fn upgrade_packages(ctx: &ExecutionContext) -> Result<()> {
    print_separator("OpenBSD Packages");

    with_sudo(ctx, |sudo| {
        ctx
            .run_type()
            .execute(sudo)
            .args(&["/usr/sbin/pkg_add", "-u"])
            .status_checked()?;
        Ok(())
    })
}
