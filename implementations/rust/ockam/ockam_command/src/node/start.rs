use clap::Args;

use colorful::Colorful;
use ockam::TcpTransport;
use ockam_api::cli_state::{StateDirTrait, StateItemTrait};

use crate::node::show::print_query_status;
use crate::node::util::{check_default, spawn_node};
use crate::node::{get_node_name, initialize_node_if_default};
use crate::util::{node_rpc, RpcBuilder};
use crate::{docs, fmt_err, CommandGlobalOpts};

const LONG_ABOUT: &str = include_str!("./static/start/long_about.txt");
const AFTER_LONG_HELP: &str = include_str!("./static/start/after_long_help.txt");

/// Start a node that was previously stopped
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = true,
    long_about = docs::about(LONG_ABOUT),
    after_long_help = docs::after_help(AFTER_LONG_HELP)
)]
pub struct StartCommand {
    /// Name of the node.
    #[arg()]
    node_name: Option<String>,

    #[arg(long, default_value = "false")]
    aws_kms: bool,
}

impl StartCommand {
    pub fn run(self, opts: CommandGlobalOpts) {
        initialize_node_if_default(&opts, &self.node_name);
        node_rpc(run_impl, (opts, self))
    }
}

async fn run_impl(
    ctx: ockam::Context,
    (opts, cmd): (CommandGlobalOpts, StartCommand),
) -> crate::Result<()> {
    let node_name = get_node_name(&opts.state, &cmd.node_name);

    let node_state = opts.state.nodes.get(&node_name)?;
    // Abort if node is already running
    if node_state.is_running() {
        let n = node_state.name();
        opts.terminal
            .stdout()
            .plain(fmt_err!(
                "The node '{n}' is already running. If you want to restart it you can call `ockam node stop {n}` and then `ockam node start {n}`"
            ))
            .write_line()?;
        return Ok(());
    }
    node_state.kill_process(false)?;
    let node_setup = node_state.config().setup();

    // Restart node
    spawn_node(
        &opts,
        node_setup.verbose, // Previously user-chosen verbosity level
        &node_name,         // The selected node name
        &node_setup.default_tcp_listener()?.addr.to_string(), // The selected node api address
        None,               // No project information available
        None,               // No trusted identities
        None,               // "
        None,               // "
        None,               // Launch config
        None,               // Authority Identity
        None,               // Credential
        None,               // Trust Context
        None,               // Project Name
    )?;

    // Print node status
    let tcp = TcpTransport::create(&ctx).await?;
    let mut rpc = RpcBuilder::new(&ctx, &opts, &node_name).tcp(&tcp)?.build();
    let is_default = check_default(&opts, &node_name);
    print_query_status(&mut rpc, &node_name, true, is_default).await?;

    Ok(())
}
