use crate::execution_context::ExecutionContext;
use crate::command::CommandExt;
use crate::terminal::print_separator;
use crate::print_warning;
use color_eyre::eyre::Result;

pub fn upgrade_openbsd(ctx: &ExecutionContext) -> Result<()> {
    print_separator("OpenBSD Update");
    if let Some(sudo) = ctx.sudo() {
        ctx
            .run_type()
            .execute(sudo)
            .args(&["/usr/sbin/sysupgrade", "-n"])
            .status_checked()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }
    Ok(())
}

pub fn upgrade_packages(ctx: &ExecutionContext) -> Result<()> {
    print_separator("OpenBSD Packages");
    if let Some(sudo) = ctx.sudo() {
        ctx
            .run_type()
            .execute(sudo)
            .args(&["/usr/sbin/pkg_add", "-u"])
            .status_checked()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }
    Ok(())
}
