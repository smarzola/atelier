use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "atelier",
    version,
    about = "Project-native runtime around Codex CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Manage one Atelier project.
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// Manage the Atelier home workspace.
    Home {
        #[command(subcommand)]
        command: HomeCommand,
    },
    /// Manage the global project registry.
    Projects {
        #[command(subcommand)]
        command: ProjectsCommand,
    },
    /// Manage person identities and person-scoped memory.
    People {
        #[command(subcommand)]
        command: PeopleCommand,
    },
    /// Manage one Atelier thread.
    Thread {
        #[command(subcommand)]
        command: ThreadCommand,
    },
    /// List Atelier threads.
    Threads {
        #[command(subcommand)]
        command: ThreadsCommand,
    },
    /// Check local Codex runtime and optional project scaffold.
    Doctor {
        /// Optional Atelier project folder path.
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Run and manage the always-alive Atelier orchestration daemon.
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Manage gateway bindings to Atelier threads.
    Gateway {
        #[command(subcommand)]
        command: GatewayCommand,
    },
    /// Manage Codex app-server pending prompts.
    Prompts {
        #[command(subcommand)]
        command: PromptsCommand,
    },
    /// Inspect Atelier jobs.
    Jobs {
        #[command(subcommand)]
        command: JobsCommand,
    },
    /// Manage Codex-native skills.
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },
    /// Manage Codex-native MCP configuration.
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
    /// Show a global dashboard across registered projects.
    Status,
    /// Build or run a Codex work invocation.
    Work {
        /// Project folder path.
        project_path: PathBuf,
        /// Atelier thread id.
        #[arg(long)]
        thread: String,
        /// Current person id/name.
        #[arg(long = "as")]
        person: String,
        /// Print command/context and do not execute Codex.
        #[arg(long)]
        dry_run: bool,
        /// Attach Codex directly to the terminal so prompts and approvals can be answered.
        #[arg(long)]
        interactive: bool,
        /// Terminate an idle managed worker after this many seconds.
        #[arg(long, default_value_t = 300)]
        idle_timeout_seconds: u64,
        /// Daemon HTTP endpoint for managed work submission.
        #[arg(long)]
        daemon_url: Option<String>,
        /// Invocation-time Codex approval policy override.
        #[arg(long)]
        approval_policy: Option<String>,
        /// Invocation-time Codex sandbox mode override.
        #[arg(long)]
        sandbox: Option<String>,
        /// Invocation-time Codex model override.
        #[arg(long)]
        model: Option<String>,
        /// Enable Codex-native web search for this invocation.
        #[arg(long)]
        search: bool,
        /// User prompt.
        prompt: String,
    },
    /// Continue an existing Codex session through Atelier.
    Continue {
        /// Project folder path.
        project_path: PathBuf,
        /// Atelier thread id.
        #[arg(long)]
        thread: String,
        /// Current person id/name.
        #[arg(long = "as")]
        person: String,
        /// Resume the most recent Codex session.
        #[arg(long)]
        last: bool,
        /// Resume a specific Codex session id.
        #[arg(long)]
        session: Option<String>,
        /// Invocation-time Codex approval policy override.
        #[arg(long)]
        approval_policy: Option<String>,
        /// Invocation-time Codex model override.
        #[arg(long)]
        model: Option<String>,
        /// User prompt.
        prompt: String,
    },
    /// List Codex session lineage recorded for an Atelier thread.
    Sessions {
        /// Project folder path.
        project_path: PathBuf,
        /// Atelier thread id.
        #[arg(long)]
        thread: String,
    },
    /// Internal managed app-server worker process.
    #[command(hide = true, name = "__managed-worker")]
    ManagedWorker {
        #[arg(long)]
        job_dir: PathBuf,
        #[arg(long)]
        project_path: PathBuf,
        #[arg(long)]
        thread: String,
        #[arg(long = "as")]
        person: String,
        #[arg(long, default_value_t = 300)]
        idle_timeout_seconds: u64,
        context: String,
    },
}

#[derive(Debug, Subcommand)]
enum ProjectCommand {
    /// Initialize an Atelier project in a folder.
    Init {
        /// Project folder path.
        path: PathBuf,
        /// Stable project name.
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum HomeCommand {
    /// Initialize the Atelier home workspace.
    Init {
        /// Home workspace folder path.
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum ProjectsCommand {
    /// Register or update a project by name.
    Add {
        /// Stable project name.
        name: String,
        /// Project folder path.
        path: PathBuf,
    },
    /// List registered projects.
    List,
}

#[derive(Debug, Subcommand)]
enum PeopleCommand {
    /// Add a person in global Atelier state.
    Add {
        /// Stable person id/name.
        id: String,
    },
    /// Manage person-scoped memory.
    Memory {
        #[command(subcommand)]
        command: PeopleMemoryCommand,
    },
}

#[derive(Debug, Subcommand)]
enum PeopleMemoryCommand {
    /// Replace a person's memory text.
    Set {
        /// Stable person id/name.
        id: String,
        /// Person memory body.
        memory: String,
    },
}

#[derive(Debug, Subcommand)]
enum DaemonCommand {
    /// Run the Atelier daemon with hosted gateway and worker supervision.
    Run {
        /// Listen address for the daemon-hosted HTTP gateway, for example 127.0.0.1:8787.
        #[arg(long, default_value = "127.0.0.1:8787")]
        listen: String,
        /// Allow listening on non-loopback addresses.
        #[arg(long)]
        allow_non_loopback: bool,
        /// Require Authorization: Bearer *** using this environment variable.
        #[arg(long)]
        auth_token_env: Option<String>,
        /// Worker supervision interval in milliseconds.
        #[arg(long, default_value_t = 5_000)]
        supervision_interval_millis: u64,
    },
}

#[derive(Debug, Subcommand)]
enum GatewayCommand {
    /// Run the generic local HTTP gateway.
    Serve {
        /// Listen address, for example 127.0.0.1:8787.
        #[arg(long, default_value = "127.0.0.1:8787")]
        listen: String,
        /// Allow listening on non-loopback addresses.
        #[arg(long)]
        allow_non_loopback: bool,
        /// Require Authorization: Bearer <token> using this environment variable.
        #[arg(long)]
        auth_token_env: Option<String>,
        /// Periodically reconcile worker state while serving the gateway.
        #[arg(long)]
        supervise_workers: bool,
        /// Worker supervision interval in milliseconds.
        #[arg(long, default_value_t = 5_000)]
        supervision_interval_millis: u64,
    },
    /// Bind an external gateway user to an Atelier person.
    BindPerson {
        /// Gateway name.
        #[arg(long)]
        gateway: String,
        /// External gateway user identifier.
        #[arg(long)]
        external_user: String,
        /// Atelier person id.
        #[arg(long)]
        person: String,
    },
    /// Bind an external gateway thread to an Atelier thread.
    Bind {
        /// Project folder path.
        project_path: PathBuf,
        /// Atelier thread id.
        #[arg(long)]
        thread: String,
        /// Gateway name.
        #[arg(long)]
        gateway: String,
        /// External gateway thread identifier.
        #[arg(long)]
        external_thread: String,
    },
    /// Resolve an external gateway thread to an Atelier thread.
    Resolve {
        /// Project folder path.
        project_path: PathBuf,
        /// Gateway name.
        #[arg(long)]
        gateway: String,
        /// External gateway thread identifier.
        #[arg(long)]
        external_thread: String,
    },
}

#[derive(Debug, Subcommand)]
enum PromptsCommand {
    /// List pending Codex prompts across registered projects.
    Inbox,
    /// List pending and resolved Codex prompts in a project.
    List {
        /// Project folder path.
        project_path: PathBuf,
    },
    /// Show one Codex prompt.
    Show {
        /// Project folder path.
        project_path: PathBuf,
        /// Atelier prompt id.
        prompt_id: String,
    },
    /// Record a response for one Codex prompt.
    Respond {
        /// Project folder path.
        project_path: PathBuf,
        /// Atelier prompt id.
        prompt_id: String,
        /// Optional text answer for user-input or elicitation prompts.
        #[arg(long)]
        text: Option<String>,
        /// Optional JSON response object to forward to Codex.
        #[arg(long)]
        json: Option<String>,
        /// Decision to record.
        decision: String,
    },
}

#[derive(Debug, Subcommand)]
enum JobsCommand {
    /// List jobs in a project.
    List {
        /// Project folder path.
        project_path: PathBuf,
    },
    /// Show one job with durable artifact paths and recent logs.
    Show {
        /// Project folder path.
        project_path: PathBuf,
        /// Job id to show.
        job_id: String,
    },
    /// Recover an idle managed job from saved context.
    Recover {
        /// Project folder path.
        project_path: PathBuf,
        /// Job id to recover.
        job_id: Option<String>,
        /// Recover all idle-timeout jobs in the project.
        #[arg(long)]
        all_idle: bool,
        /// Recover all worker-lost jobs in the project.
        #[arg(long)]
        all_worker_lost: bool,
        /// Terminate the recovered worker after this many idle seconds.
        #[arg(long, default_value_t = 300)]
        idle_timeout_seconds: u64,
    },
}

#[derive(Debug, Subcommand)]
enum SkillCommand {
    /// Add a skill to a project using Codex-native `.agents/skills` layout.
    Add {
        #[command(subcommand)]
        command: SkillAddCommand,
    },
}

#[derive(Debug, Subcommand)]
enum SkillAddCommand {
    /// Add a project-local skill from a folder.
    Project {
        /// Project folder path.
        project_path: PathBuf,
        /// Source skill folder path.
        source_path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum McpCommand {
    /// Add an MCP server.
    Add {
        #[command(subcommand)]
        command: McpAddCommand,
    },
}

#[derive(Debug, Subcommand)]
enum McpAddCommand {
    /// Add a project-local MCP server to `.codex/config.toml`.
    Project {
        /// Project folder path.
        project_path: PathBuf,
        /// MCP server name.
        name: String,
        /// MCP command and arguments after `--`.
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ThreadCommand {
    /// Create a new Atelier thread in a project.
    New {
        /// Project folder path.
        project_path: PathBuf,
        /// Thread title.
        title: String,
        /// Print only the created thread id.
        #[arg(long)]
        porcelain: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ThreadsCommand {
    /// List Atelier threads in a project.
    List {
        /// Project folder path.
        project_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Project { command } => match command {
            ProjectCommand::Init { path, name } => {
                atelier_core::project::init_project(&path, &name)?;
                let project = atelier_core::registry::add_project(&name, &path)?;
                println!(
                    "Initialized Atelier project '{}' at {}",
                    name,
                    path.display()
                );
                println!(
                    "Registered project {} at {}",
                    project.name,
                    project.path.display()
                );
            }
        },
        Command::Home { command } => match command {
            HomeCommand::Init { path } => {
                init_home_workspace(&path)?;
                println!("Initialized Atelier home at {}", path.display());
            }
        },
        Command::Projects { command } => match command {
            ProjectsCommand::Add { name, path } => {
                let project = atelier_core::registry::add_project(&name, &path)?;
                println!(
                    "Added project {} at {}",
                    project.name,
                    project.path.display()
                );
            }
            ProjectsCommand::List => {
                for project in atelier_core::registry::list_projects()? {
                    println!("{}\t{}", project.name, project.path.display());
                }
            }
        },
        Command::People { command } => match command {
            PeopleCommand::Add { id } => {
                let memory_path = atelier_core::people::add_person(&id)?;
                println!("Added person {id} at {}", memory_path.display());
            }
            PeopleCommand::Memory { command } => match command {
                PeopleMemoryCommand::Set { id, memory } => {
                    let memory_path = atelier_core::people::set_person_memory(&id, &memory)?;
                    println!("Updated memory for {id} at {}", memory_path.display());
                }
            },
        },
        Command::Thread { command } => match command {
            ThreadCommand::New {
                project_path,
                title,
                porcelain,
            } => {
                let thread = atelier_core::thread::create_thread(&project_path, &title)?;
                if porcelain {
                    println!("{}", thread.id);
                } else {
                    println!("Created thread '{}' ({})", thread.title, thread.id);
                }
            }
        },
        Command::Threads { command } => match command {
            ThreadsCommand::List { project_path } => {
                for thread in atelier_core::thread::list_threads(&project_path)? {
                    println!("{}\t{}\t{}", thread.id, thread.status, thread.title);
                }
            }
        },
        Command::Doctor { project } => {
            let report = atelier_core::doctor::run_doctor(project.as_deref());
            for check in &report.checks {
                println!(
                    "{}: {} — {}",
                    check.name,
                    check.status.as_str(),
                    check.detail
                );
            }
            if !report.is_ok() {
                std::process::exit(1);
            }
        }
        Command::Daemon { command } => match command {
            DaemonCommand::Run {
                listen,
                allow_non_loopback,
                auth_token_env,
                supervision_interval_millis,
            } => {
                serve_gateway(
                    &listen,
                    allow_non_loopback,
                    auth_token_env,
                    true,
                    supervision_interval_millis,
                )?;
            }
        },
        Command::Gateway { command } => match command {
            GatewayCommand::Serve {
                listen,
                allow_non_loopback,
                auth_token_env,
                supervise_workers,
                supervision_interval_millis,
            } => {
                serve_gateway(
                    &listen,
                    allow_non_loopback,
                    auth_token_env,
                    supervise_workers,
                    supervision_interval_millis,
                )?;
            }
            GatewayCommand::BindPerson {
                gateway,
                external_user,
                person,
            } => {
                let binding =
                    atelier_core::gateway::bind_person(&gateway, &external_user, &person)?;
                println!(
                    "Bound {}:{} to {}",
                    binding.gateway, binding.external_user, binding.person
                );
            }
            GatewayCommand::Bind {
                project_path,
                thread,
                gateway,
                external_thread,
            } => {
                let binding = atelier_core::gateway::bind_thread(
                    &project_path,
                    &thread,
                    &gateway,
                    &external_thread,
                )?;
                println!(
                    "Bound {}:{} to {}",
                    binding.gateway, binding.external_thread, binding.thread_id
                );
            }
            GatewayCommand::Resolve {
                project_path,
                gateway,
                external_thread,
            } => {
                if let Some(binding) = atelier_core::gateway::resolve_thread(
                    &project_path,
                    &gateway,
                    &external_thread,
                )? {
                    println!("{}", binding.thread_id);
                } else {
                    std::process::exit(1);
                }
            }
        },
        Command::Prompts { command } => match command {
            PromptsCommand::Inbox => {
                print_prompt_inbox()?;
            }
            PromptsCommand::List { project_path } => {
                let project_path = resolve_project_arg(&project_path)?;
                for (job_id, prompt) in list_prompts(&project_path)? {
                    println!(
                        "{}\t{:?}\t{}\t{}",
                        prompt.id, prompt.status, prompt.summary, job_id
                    );
                }
            }
            PromptsCommand::Show {
                project_path,
                prompt_id,
            } => {
                let project_path = resolve_project_arg(&project_path)?;
                let (_job_dir, prompt) = find_prompt(&project_path, &prompt_id)?;
                println!("Prompt: {}", prompt.id);
                println!("Status: {:?}", prompt.status);
                println!("Method: {}", prompt.method);
                println!("Summary: {}", prompt.summary);
                if !prompt.available_decisions.is_empty() {
                    println!(
                        "Decision options: {}",
                        prompt.available_decisions.join(", ")
                    );
                }
            }
            PromptsCommand::Respond {
                project_path,
                prompt_id,
                text,
                json,
                decision,
            } => {
                let project_path = resolve_project_arg(&project_path)?;
                let (job_dir, mut prompt) = find_prompt(&project_path, &prompt_id)?;
                let response = build_prompt_response(&prompt, &decision, text, json)?;
                prompt.status = atelier_core::codex_app_server::PendingPromptStatus::Resolved;
                let prompt_path = job_dir.join("prompts").join(format!("{}.json", prompt.id));
                std::fs::write(
                    prompt_path,
                    serde_json::to_string_pretty(&prompt).context("serialize prompt")?,
                )?;
                let responses_dir = job_dir.join("responses");
                std::fs::create_dir_all(&responses_dir)?;
                std::fs::write(
                    responses_dir.join(format!("{}.json", prompt.id)),
                    serde_json::to_string_pretty(&response)?,
                )?;
                println!("Recorded response {decision} for {prompt_id}");
            }
        },
        Command::Jobs { command } => match command {
            JobsCommand::List { project_path } => {
                let project_path = resolve_project_arg(&project_path)?;
                for status in list_jobs(&project_path)? {
                    println!("{}\t{}\t{}", status.id, status.status, status.thread_id);
                }
            }
            JobsCommand::Show {
                project_path,
                job_id,
            } => {
                let project_path = resolve_project_arg(&project_path)?;
                show_job(&project_path, &job_id)?;
            }
            JobsCommand::Recover {
                project_path,
                job_id,
                all_idle,
                all_worker_lost,
                idle_timeout_seconds,
            } => {
                let project_path = resolve_project_arg(&project_path)?;
                if all_idle || all_worker_lost {
                    let wanted_status = if all_idle {
                        "idle-timeout"
                    } else {
                        "worker-lost"
                    };
                    let mut recovered = 0usize;
                    for status in list_jobs(&project_path)? {
                        if status.status == wanted_status {
                            recover_job(&project_path, &status.id, idle_timeout_seconds)?;
                            println!("Recovered job: {}", status.id);
                            recovered += 1;
                        }
                    }
                    println!("Recovered {recovered} jobs");
                } else {
                    let job_id = job_id
                        .context("recover requires <job-id>, --all-idle, or --all-worker-lost")?;
                    recover_job(&project_path, &job_id, idle_timeout_seconds)?;
                    println!("Recovered job: {job_id}");
                }
            }
        },
        Command::Skill { command } => match command {
            SkillCommand::Add { command } => match command {
                SkillAddCommand::Project {
                    project_path,
                    source_path,
                } => {
                    let skill_name =
                        atelier_core::codex_native::add_project_skill(&project_path, &source_path)?;
                    println!("Added project skill {skill_name}");
                }
            },
        },
        Command::Mcp { command } => match command {
            McpCommand::Add { command } => match command {
                McpAddCommand::Project {
                    project_path,
                    name,
                    command,
                } => {
                    let (binary, args) = command
                        .split_first()
                        .ok_or_else(|| anyhow::anyhow!("mcp command is required"))?;
                    atelier_core::codex_native::add_project_mcp_server(
                        &project_path,
                        &name,
                        binary,
                        args,
                    )?;
                    println!("Added project MCP server {name}");
                }
            },
        },
        Command::Status => {
            print_global_status()?;
        }
        Command::Work {
            project_path,
            thread,
            person,
            dry_run,
            interactive,
            idle_timeout_seconds,
            daemon_url,
            approval_policy,
            sandbox,
            model,
            search,
            prompt,
        } => {
            let project_arg = project_path.to_string_lossy().to_string();
            let project_path = resolve_project_arg(&project_path)?;
            let policy = atelier_core::codex::CodexPolicy {
                approval_policy,
                sandbox,
                model,
                search,
            };
            let context = build_context(&person, &thread, &prompt)?;

            if dry_run {
                let job = atelier_core::job::create_job(
                    &project_path,
                    &thread,
                    &person,
                    &prompt,
                    &context,
                    true,
                )?;
                let invocation = atelier_core::codex::CodexInvocation::with_policy(
                    &project_path,
                    context.clone(),
                    policy,
                );
                println!("Job: {}", job.id);
                println!("Job directory: {}", job.dir.display());
                println!("Would run: {}", invocation.display_command());
                println!("\n{}", invocation.prompt);
            } else if interactive {
                let job = atelier_core::job::create_job(
                    &project_path,
                    &thread,
                    &person,
                    &prompt,
                    &context,
                    false,
                )?;
                let invocation = atelier_core::codex::CodexInvocation::with_policy(
                    &project_path,
                    context.clone(),
                    policy,
                );
                let output = invocation.run_interactive()?;
                finish_job(&job, &thread, &person, output)?;
            } else {
                let response = submit_managed_work_to_daemon(
                    &daemon_url.unwrap_or_else(default_daemon_url),
                    &project_arg,
                    &thread,
                    &person,
                    &prompt,
                    idle_timeout_seconds,
                )?;
                println!("Job: {}", response.job_id);
                println!("Job directory: {}", response.job_dir.display());
            }
        }
        Command::Continue {
            project_path,
            thread,
            person,
            last,
            session,
            approval_policy,
            model,
            prompt,
        } => {
            let project_path = resolve_project_arg(&project_path)?;
            let policy = atelier_core::codex::CodexPolicy {
                approval_policy,
                sandbox: None,
                model,
                search: false,
            };
            let context = build_context(&person, &thread, &prompt)?;
            let job = atelier_core::job::create_job(
                &project_path,
                &thread,
                &person,
                &prompt,
                &context,
                false,
            )?;
            let invocation = if last {
                atelier_core::codex::CodexResumeInvocation::last_with_policy(context, policy)
            } else if let Some(session) = session {
                atelier_core::codex::CodexResumeInvocation::session_with_policy(
                    session, context, policy,
                )
            } else {
                anyhow::bail!("continue requires --last or --session <id>");
            };
            let output = invocation.run()?;
            finish_job(&job, &thread, &person, output)?;
        }
        Command::Sessions {
            project_path,
            thread,
        } => {
            let project_path = resolve_project_arg(&project_path)?;
            print!(
                "{}",
                atelier_core::thread::codex_session_lineage(&project_path, &thread)?
            );
        }
        Command::ManagedWorker {
            job_dir,
            project_path,
            thread,
            person,
            idle_timeout_seconds,
            context,
        } => {
            run_managed_worker(
                &job_dir,
                &project_path,
                &thread,
                &person,
                &context,
                idle_timeout_seconds,
            )?;
        }
    }

    Ok(())
}

fn init_home_workspace(path: &Path) -> Result<()> {
    atelier_core::project::init_project(path, "home")?;
    let skills_dir = path.join(".agents/skills");
    std::fs::create_dir_all(&skills_dir).context("create home skills directory")?;
    write_home_skill(
        &skills_dir.join("route-project/SKILL.md"),
        "route-project",
        "Route requests from the home workspace to the right Atelier project.",
    )?;
    write_home_skill(
        &skills_dir.join("inspect-runtime/SKILL.md"),
        "inspect-runtime",
        "Inspect Atelier jobs, prompts, status, and recoverable work.",
    )?;
    atelier_core::registry::add_project("home", path)?;
    Ok(())
}

fn write_home_skill(path: &Path, name: &str, description: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create home skill directory")?;
    }
    std::fs::write(
        path,
        format!(
            "---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n\n{description}\n\nUse project files as the source of truth. Person memory belongs outside projects.\n"
        ),
    )
    .context("write home skill")
}

fn print_prompt_inbox() -> Result<()> {
    for project in atelier_core::registry::list_projects()? {
        for (job_id, prompt) in list_prompts(&project.path)? {
            if prompt.status == atelier_core::codex_app_server::PendingPromptStatus::Pending {
                println!(
                    "{}\t{}\t{}\t{}",
                    project.name, job_id, prompt.id, prompt.summary
                );
            }
        }
    }
    Ok(())
}

fn recover_job(project_path: &Path, job_id: &str, idle_timeout_seconds: u64) -> Result<()> {
    let job_dir = project_path.join(".atelier/jobs").join(job_id);
    let status: atelier_core::job::JobStatus = serde_json::from_str(
        &std::fs::read_to_string(job_dir.join("status.json")).context("read job status")?,
    )
    .context("parse job status")?;
    let context =
        std::fs::read_to_string(job_dir.join("context.md")).context("read job context")?;
    run_managed_worker(
        &job_dir,
        project_path,
        &status.thread_id,
        &status.person,
        &context,
        idle_timeout_seconds,
    )
}

fn print_global_status() -> Result<()> {
    let projects = atelier_core::registry::list_projects()?;
    let mut all_jobs = Vec::new();
    let mut waiting_prompts = 0usize;
    for project in &projects {
        for status in list_jobs(&project.path)? {
            if status.status == "waiting-for-prompt" {
                waiting_prompts += list_prompts(&project.path)?
                    .into_iter()
                    .filter(|(_job_id, prompt)| {
                        prompt.status
                            == atelier_core::codex_app_server::PendingPromptStatus::Pending
                    })
                    .count();
            }
            all_jobs.push((project.name.clone(), status));
        }
    }
    let active_jobs = all_jobs
        .iter()
        .filter(|(_project, status)| {
            status.status == "running" || status.status == "waiting-for-prompt"
        })
        .count();
    let worker_lost_jobs = all_jobs
        .iter()
        .filter(|(_project, status)| status.status == "worker-lost")
        .count();
    let idle_timeout_jobs = all_jobs
        .iter()
        .filter(|(_project, status)| status.status == "idle-timeout")
        .count();

    println!("Projects: {}", projects.len());
    println!("Active jobs: {active_jobs}");
    println!("Waiting prompts: {waiting_prompts}");
    println!("Worker-lost jobs: {worker_lost_jobs}");
    println!("Idle-timeout jobs: {idle_timeout_jobs}");
    for (project_name, status) in all_jobs {
        println!(
            "{}\t{}\t{}\t{}",
            project_name, status.id, status.status, status.thread_id
        );
    }
    Ok(())
}

fn resolve_project_arg(project: &Path) -> Result<PathBuf> {
    atelier_core::registry::resolve_project_path(project.to_string_lossy().as_ref())
}

#[derive(Debug, serde::Deserialize)]
struct GatewayPromptResponseRequest {
    project: String,
    prompt_id: String,
    decision: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    json: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct GatewayProjectCreateRequest {
    name: String,
    path: PathBuf,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DaemonWorkRequest {
    project: String,
    thread: String,
    person: String,
    text: String,
    #[serde(default = "default_idle_timeout_seconds")]
    idle_timeout_seconds: u64,
}

#[derive(Debug, serde::Deserialize)]
struct DaemonWorkResponse {
    job_id: String,
    job_dir: PathBuf,
}

fn default_idle_timeout_seconds() -> u64 {
    300
}

#[derive(Debug, Clone)]
struct GatewayAuth {
    bearer_token: Option<String>,
    telegram: TelegramConfig,
}

#[derive(Debug, Clone)]
struct TelegramConfig {
    bot_token: Option<String>,
    api_base: String,
    webhook_url: Option<String>,
    webhook_secret: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TelegramSendMessageRequest {
    chat_id: serde_json::Value,
    text: String,
    message_thread_id: Option<serde_json::Value>,
    reply_to_message_id: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct TelegramUpdateMetadata {
    chat_id: String,
    message_thread_id: Option<String>,
}

fn serve_gateway(
    listen: &str,
    allow_non_loopback: bool,
    auth_token_env: Option<String>,
    supervise_workers: bool,
    supervision_interval_millis: u64,
) -> Result<()> {
    validate_gateway_listen_address(listen, allow_non_loopback)?;
    let auth = GatewayAuth {
        bearer_token: match auth_token_env {
            Some(name) => Some(
                std::env::var(&name)
                    .with_context(|| format!("auth token env var is not set: {name}"))?,
            ),
            None => None,
        },
        telegram: load_telegram_config(),
    };
    let listener = TcpListener::bind(listen).with_context(|| format!("bind gateway {listen}"))?;
    let supervisor_stop = if supervise_workers {
        Some(start_worker_supervisor(supervision_interval_millis))
    } else {
        None
    };
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_gateway_stream(&mut stream, &auth) {
                    let _ = write_json_response(
                        &mut stream,
                        500,
                        serde_json::json!({"error": error.to_string()}),
                    );
                }
            }
            Err(error) => return Err(error).context("accept gateway connection"),
        }
    }
    drop(supervisor_stop);
    Ok(())
}

fn start_worker_supervisor(interval_millis: u64) -> Arc<std::sync::atomic::AtomicBool> {
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let interval = Duration::from_millis(interval_millis.max(10));
    std::thread::spawn(move || {
        while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = reconcile_registered_project_workers();
            std::thread::sleep(interval);
        }
    });
    stop
}

fn reconcile_registered_project_workers() -> Result<()> {
    for project in atelier_core::registry::list_projects()? {
        let _ = list_jobs(&project.path)?;
    }
    Ok(())
}

fn validate_gateway_listen_address(listen: &str, allow_non_loopback: bool) -> Result<()> {
    if allow_non_loopback {
        return Ok(());
    }
    let addr: std::net::SocketAddr = listen
        .parse()
        .with_context(|| format!("parse listen address {listen}"))?;
    if !addr.ip().is_loopback() {
        anyhow::bail!(
            "refusing to listen on non-loopback address {listen}; pass --allow-non-loopback to opt in"
        );
    }
    Ok(())
}

fn handle_gateway_stream(stream: &mut TcpStream, auth: &GatewayAuth) -> Result<()> {
    let request = read_http_request(stream)?;
    if !gateway_request_is_authorized(&request, auth) {
        write_json_response(stream, 401, serde_json::json!({"error":"unauthorized"}))?;
        return Ok(());
    }
    if !telegram_update_request_is_authorized(&request, &auth.telegram) {
        write_json_response(stream, 401, serde_json::json!({"error":"unauthorized"}))?;
        return Ok(());
    }
    let method = request.method;
    let path = request.path;
    let body = request.body;
    let response = match (method.as_str(), path.as_str()) {
        ("GET", "/health") => serde_json::json!({"status":"ok"}),
        ("GET", "/status") => gateway_status_json()?,
        ("GET", "/jobs") => gateway_jobs_json()?,
        ("GET", "/prompts") => gateway_prompts_json()?,
        ("GET", "/projects") => gateway_projects_json()?,
        ("POST", "/projects") => {
            let request: GatewayProjectCreateRequest = serde_json::from_str(&body)?;
            atelier_core::project::init_project(&request.path, &request.name)?;
            let project = atelier_core::registry::add_project(&request.name, &request.path)?;
            append_gateway_audit_event(serde_json::json!({
                "action": "project_created",
                "project": project.name,
                "project_path": project.path,
                "result": "created"
            }))?;
            serde_json::json!({"status":"created","project":{"name":project.name,"path":project.path}})
        }
        ("POST", "/work") => {
            let request: DaemonWorkRequest = serde_json::from_str(&body)?;
            start_daemon_work(request)?
        }
        ("POST", "/prompts/respond") => {
            let request: GatewayPromptResponseRequest = serde_json::from_str(&body)?;
            let project_path = resolve_project_arg(Path::new(&request.project))?;
            respond_to_prompt(
                &project_path,
                &request.prompt_id,
                &request.decision,
                request.text,
                request.json,
            )?;
            append_gateway_audit_event(serde_json::json!({
                "action": "prompt_response",
                "project": request.project,
                "project_path": project_path,
                "prompt_id": request.prompt_id,
                "decision": request.decision,
                "result": "recorded"
            }))?;
            serde_json::json!({"status":"recorded","prompt_id":request.prompt_id})
        }
        ("POST", "/events/message") => {
            let event: atelier_core::gateway::GatewayMessageEvent = serde_json::from_str(&body)?;
            handle_gateway_message_event(event)?
        }
        ("POST", "/adapters/telegram/update") => {
            let metadata = telegram_update_metadata(&body)?;
            let event = telegram_update_to_gateway_event(&body)?;
            let response = handle_gateway_message_event(event)?;
            if let Some(job_id) = response.get("job_id").and_then(serde_json::Value::as_str) {
                let _ = telegram_acknowledge_job_start(&auth.telegram, &metadata, job_id);
            }
            response
        }
        ("POST", "/adapters/telegram/webhook/setup") => telegram_set_webhook(&auth.telegram)?,
        ("POST", "/adapters/telegram/send-message") => {
            let request: TelegramSendMessageRequest = serde_json::from_str(&body)?;
            telegram_send_message(&auth.telegram, request)?
        }
        _ => {
            write_json_response(stream, 404, serde_json::json!({"error":"not found"}))?;
            return Ok(());
        }
    };
    write_json_response(stream, 200, response)
}

fn load_telegram_config() -> TelegramConfig {
    TelegramConfig {
        bot_token: std::env::var("ATELIER_TELEGRAM_BOT_TOKEN").ok(),
        api_base: std::env::var("ATELIER_TELEGRAM_API_BASE")
            .unwrap_or_else(|_| "https://api.telegram.org".to_string()),
        webhook_url: std::env::var("ATELIER_TELEGRAM_WEBHOOK_URL").ok(),
        webhook_secret: std::env::var("ATELIER_TELEGRAM_WEBHOOK_SECRET").ok(),
    }
}

fn telegram_update_request_is_authorized(request: &HttpRequest, config: &TelegramConfig) -> bool {
    if request.path != "/adapters/telegram/update" {
        return true;
    }
    let Some(expected_secret) = &config.webhook_secret else {
        return true;
    };
    request
        .headers
        .iter()
        .find(|(name, _value)| name.eq_ignore_ascii_case("x-telegram-bot-api-secret-token"))
        .map(|(_name, value)| value == expected_secret)
        .unwrap_or(false)
}

fn telegram_set_webhook(config: &TelegramConfig) -> Result<serde_json::Value> {
    let url = config
        .webhook_url
        .as_deref()
        .context("ATELIER_TELEGRAM_WEBHOOK_URL is required to set Telegram webhook")?;
    let mut body = serde_json::json!({"url": url});
    if let Some(secret) = &config.webhook_secret {
        body["secret_token"] = serde_json::json!(secret);
    }
    let result = telegram_bot_api_json(config, "setWebhook", body)?;
    append_gateway_audit_event(serde_json::json!({
        "action": "telegram_webhook_setup",
        "result": "configured"
    }))?;
    Ok(serde_json::json!({"status":"configured","result":result}))
}

fn telegram_send_message(
    config: &TelegramConfig,
    request: TelegramSendMessageRequest,
) -> Result<serde_json::Value> {
    let result = telegram_send_message_body(
        config,
        request.chat_id,
        request.message_thread_id,
        request.reply_to_message_id,
        request.text,
    )?;
    append_gateway_audit_event(serde_json::json!({
        "action": "telegram_send_message",
        "result": "sent"
    }))?;
    Ok(serde_json::json!({"status":"sent","result":result}))
}

fn telegram_acknowledge_job_start(
    config: &TelegramConfig,
    metadata: &TelegramUpdateMetadata,
    job_id: &str,
) -> Result<()> {
    telegram_send_message_body(
        config,
        serde_json::json!(metadata.chat_id),
        metadata
            .message_thread_id
            .as_ref()
            .map(|thread_id| serde_json::json!(thread_id)),
        None,
        format!("Atelier started job {job_id}."),
    )?;
    append_gateway_audit_event(serde_json::json!({
        "action": "telegram_job_acknowledgement",
        "job_id": job_id,
        "result": "sent"
    }))?;
    Ok(())
}

fn telegram_send_message_body(
    config: &TelegramConfig,
    chat_id: serde_json::Value,
    message_thread_id: Option<serde_json::Value>,
    reply_to_message_id: Option<serde_json::Value>,
    text: String,
) -> Result<serde_json::Value> {
    let mut body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
    });
    if let Some(message_thread_id) = message_thread_id {
        body["message_thread_id"] = message_thread_id;
    }
    if let Some(reply_to_message_id) = reply_to_message_id {
        body["reply_to_message_id"] = reply_to_message_id;
    }
    telegram_bot_api_json(config, "sendMessage", body)
}

fn telegram_bot_api_json(
    config: &TelegramConfig,
    method: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value> {
    let token = config
        .bot_token
        .as_deref()
        .context("ATELIER_TELEGRAM_BOT_TOKEN is required for Telegram Bot API calls")?;
    let url = telegram_bot_api_url(&config.api_base, token, method);
    let result: serde_json::Value = reqwest::blocking::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .with_context(|| format!("send Telegram Bot API request to {url}"))?
        .error_for_status()
        .with_context(|| format!("Telegram Bot API returned an HTTP error for {method}"))?
        .json()
        .context("parse Telegram Bot API response")?;
    if result.get("ok").and_then(serde_json::Value::as_bool) == Some(false) {
        let description = result
            .get("description")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Telegram Bot API returned ok=false");
        anyhow::bail!("{description}");
    }
    Ok(result)
}

fn telegram_bot_api_url(base_url: &str, token: &str, method: &str) -> String {
    format!("{}/bot{}/{}", base_url.trim_end_matches('/'), token, method)
}

#[cfg(test)]
mod telegram_tests {
    use super::telegram_bot_api_url;

    #[test]
    fn telegram_bot_api_url_defaults_to_https_host_shape() {
        assert_eq!(
            telegram_bot_api_url("https://api.telegram.org", "example-token", "sendMessage"),
            "https://api.telegram.org/botexample-token/sendMessage"
        );
    }
}

fn default_daemon_url() -> String {
    std::env::var("ATELIER_DAEMON_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".to_string())
}

fn submit_managed_work_to_daemon(
    daemon_url: &str,
    project: &str,
    thread: &str,
    person: &str,
    text: &str,
    idle_timeout_seconds: u64,
) -> Result<DaemonWorkResponse> {
    let request = DaemonWorkRequest {
        project: project.to_string(),
        thread: thread.to_string(),
        person: person.to_string(),
        text: text.to_string(),
        idle_timeout_seconds,
    };
    let body = serde_json::to_string(&request).context("serialize daemon work request")?;
    let response = daemon_http_request(daemon_url, "POST", "/work", &body).with_context(|| {
        format!(
            "work requires a running Atelier daemon at {daemon_url}; start one with `atelier daemon run`"
        )
    })?;
    serde_json::from_str(&response).context("parse daemon work response")
}

fn daemon_http_request(base_url: &str, method: &str, path: &str, body: &str) -> Result<String> {
    let (host, port) = parse_loopback_http_url(base_url)?;
    let mut stream = TcpStream::connect((host.as_str(), port))
        .with_context(|| format!("connect daemon at {base_url}"))?;
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .context("write daemon request")?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .context("read daemon response")?;
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .context("parse daemon response status")?;
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_string();
    if !(200..300).contains(&status) {
        anyhow::bail!("daemon returned HTTP {status}: {body}");
    }
    Ok(body)
}

fn parse_loopback_http_url(base_url: &str) -> Result<(String, u16)> {
    let remainder = base_url
        .strip_prefix("http://")
        .context("daemon URL must start with http://")?;
    let authority = remainder.split('/').next().unwrap_or(remainder);
    let (host, port) = authority
        .rsplit_once(':')
        .context("daemon URL must include host and port")?;
    let port = port
        .parse::<u16>()
        .with_context(|| format!("parse daemon URL port in {base_url}"))?;
    Ok((host.to_string(), port))
}

fn start_daemon_work(request: DaemonWorkRequest) -> Result<serde_json::Value> {
    let project_path = resolve_project_arg(Path::new(&request.project))?;
    ensure_project_writer_available(&project_path)?;
    let context = build_context(&request.person, &request.thread, &request.text)?;
    let job = atelier_core::job::create_job(
        &project_path,
        &request.thread,
        &request.person,
        &request.text,
        &context,
        false,
    )?;
    run_managed_app_server_job(
        &job,
        &project_path,
        &request.thread,
        &request.person,
        &context,
        request.idle_timeout_seconds,
    )?;
    append_gateway_audit_event(serde_json::json!({
        "action": "work_started",
        "project": request.project,
        "project_path": project_path,
        "thread": request.thread,
        "person": request.person,
        "job_id": job.id,
        "result": "started"
    }))?;
    Ok(serde_json::json!({
        "status":"started",
        "job_id":job.id,
        "job_dir":job.dir,
        "project":request.project,
        "thread":request.thread,
        "person":request.person
    }))
}

fn handle_gateway_message_event(
    event: atelier_core::gateway::GatewayMessageEvent,
) -> Result<serde_json::Value> {
    let (project, project_path, thread) = resolve_gateway_project_thread(&event)?;
    let person = resolve_gateway_person(&event)?;
    ensure_project_writer_available(&project_path)?;
    let context = build_context(&person, &thread, &event.text)?;
    let job = atelier_core::job::create_job(
        &project_path,
        &thread,
        &person,
        &event.text,
        &context,
        false,
    )?;
    run_managed_app_server_job(&job, &project_path, &thread, &person, &context, 300)?;
    append_gateway_audit_event(serde_json::json!({
        "action": "message_started",
        "gateway": event.gateway,
        "external_thread": event.external_thread,
        "external_user": event.external_user,
        "project": project,
        "project_path": project_path,
        "thread": thread,
        "person": person,
        "job_id": job.id,
        "result": "started"
    }))?;
    Ok(
        serde_json::json!({"status":"started","job_id":job.id,"job_dir":job.dir,"project":project,"thread":thread,"person":person}),
    )
}

fn append_gateway_audit_event(mut event: serde_json::Value) -> Result<()> {
    if let Some(object) = event.as_object_mut() {
        object.insert(
            "timestamp_unix_seconds".to_string(),
            serde_json::json!(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        );
    }
    let audit_path = atelier_core::people::atelier_home().join("gateway/audit.jsonl");
    if let Some(parent) = audit_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .with_context(|| format!("open {}", audit_path.display()))?;
    writeln!(file, "{}", serde_json::to_string(&event)?)
        .with_context(|| format!("write {}", audit_path.display()))
}

fn telegram_update_to_gateway_event(
    body: &str,
) -> Result<atelier_core::gateway::GatewayMessageEvent> {
    let update: serde_json::Value = serde_json::from_str(body).context("parse Telegram update")?;
    let message = telegram_update_message(&update)?;
    let text = message
        .get("text")
        .or_else(|| message.get("caption"))
        .and_then(serde_json::Value::as_str)
        .context("Telegram message update missing text")?;
    let metadata = telegram_message_metadata(message)?;
    let external_thread = if let Some(topic_id) = &metadata.message_thread_id {
        format!("chat:{}:topic:{}", metadata.chat_id, topic_id)
    } else {
        format!("chat:{}", metadata.chat_id)
    };
    let external_user = message
        .get("from")
        .and_then(|from| from.get("id"))
        .and_then(json_id_as_string)
        .context("Telegram message update missing sender id")?;
    Ok(atelier_core::gateway::GatewayMessageEvent {
        gateway: "telegram".to_string(),
        external_thread: Some(external_thread),
        external_user: Some(external_user),
        project: None,
        thread: None,
        person: None,
        text: text.to_string(),
    })
}

fn telegram_update_metadata(body: &str) -> Result<TelegramUpdateMetadata> {
    let update: serde_json::Value = serde_json::from_str(body).context("parse Telegram update")?;
    telegram_message_metadata(telegram_update_message(&update)?)
}

fn telegram_update_message(update: &serde_json::Value) -> Result<&serde_json::Value> {
    update
        .get("message")
        .or_else(|| update.get("edited_message"))
        .context("Telegram update missing message")
}

fn telegram_message_metadata(message: &serde_json::Value) -> Result<TelegramUpdateMetadata> {
    let chat_id = message
        .get("chat")
        .and_then(|chat| chat.get("id"))
        .and_then(json_id_as_string)
        .context("Telegram message update missing chat id")?;
    let message_thread_id = message.get("message_thread_id").and_then(json_id_as_string);
    Ok(TelegramUpdateMetadata {
        chat_id,
        message_thread_id,
    })
}

fn json_id_as_string(value: &serde_json::Value) -> Option<String> {
    if let Some(id) = value.as_i64() {
        Some(id.to_string())
    } else if let Some(id) = value.as_u64() {
        Some(id.to_string())
    } else {
        value.as_str().map(ToString::to_string)
    }
}

fn resolve_gateway_person(event: &atelier_core::gateway::GatewayMessageEvent) -> Result<String> {
    if let Some(person) = &event.person {
        return Ok(person.clone());
    }
    let external_user = event
        .external_user
        .as_deref()
        .context("message event requires person or external_user")?;
    atelier_core::gateway::resolve_person(&event.gateway, external_user)?
        .map(|binding| binding.person)
        .context("no person binding found for gateway user")
}

fn resolve_gateway_project_thread(
    event: &atelier_core::gateway::GatewayMessageEvent,
) -> Result<(String, PathBuf, String)> {
    if let (Some(project), Some(thread)) = (&event.project, &event.thread) {
        let project_path = resolve_project_arg(Path::new(project))?;
        return Ok((project.clone(), project_path, thread.clone()));
    }
    let external_thread = event
        .external_thread
        .as_deref()
        .context("message event requires project/thread or external_thread")?;
    for project in atelier_core::registry::list_projects()? {
        if let Some(binding) =
            atelier_core::gateway::resolve_thread(&project.path, &event.gateway, external_thread)?
        {
            return Ok((project.name, project.path, binding.thread_id));
        }
    }
    anyhow::bail!("no thread binding found for gateway thread")
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    authorization: Option<String>,
    headers: Vec<(String, String)>,
    body: String,
}

fn gateway_request_is_authorized(request: &HttpRequest, auth: &GatewayAuth) -> bool {
    let Some(expected_token) = &auth.bearer_token else {
        return true;
    };
    request.authorization.as_deref() == Some(&format!("Bearer {expected_token}"))
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    loop {
        let bytes = stream.read(&mut temp).context("read http request")?;
        if bytes == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..bytes]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    let header_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .context("http request missing header terminator")?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let request_line = headers.lines().next().context("missing request line")?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .context("missing http method")?
        .to_string();
    let path = request_parts
        .next()
        .context("missing http path")?
        .to_string();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    let authorization = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("authorization") {
            Some(value.trim().to_string())
        } else {
            None
        }
    });
    let parsed_headers = headers
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_string(), value.trim().to_string()))
        })
        .collect();
    while buffer.len() < header_end + content_length {
        let bytes = stream.read(&mut temp).context("read http body")?;
        if bytes == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..bytes]);
    }
    let body =
        String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).to_string();
    Ok(HttpRequest {
        method,
        path,
        authorization,
        headers: parsed_headers,
        body,
    })
}

fn write_json_response(
    stream: &mut TcpStream,
    status: u16,
    value: serde_json::Value,
) -> Result<()> {
    let reason = if status == 200 {
        "OK"
    } else if status == 401 {
        "Unauthorized"
    } else if status == 404 {
        "Not Found"
    } else {
        "Internal Server Error"
    };
    let body = serde_json::to_string(&value)?;
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    Ok(())
}

fn gateway_status_json() -> Result<serde_json::Value> {
    let projects = atelier_core::registry::list_projects()?;
    let mut jobs = Vec::new();
    let mut waiting_prompts = 0usize;
    for project in &projects {
        for status in list_jobs(&project.path)? {
            if status.status == "waiting-for-prompt" {
                waiting_prompts += list_prompts(&project.path)?
                    .into_iter()
                    .filter(|(_job_id, prompt)| {
                        prompt.status
                            == atelier_core::codex_app_server::PendingPromptStatus::Pending
                    })
                    .count();
            }
            jobs.push(status);
        }
    }
    Ok(serde_json::json!({
        "projects": projects.len(),
        "active_jobs": jobs.iter().filter(|status| status.status == "running" || status.status == "waiting-for-prompt").count(),
        "waiting_prompts": waiting_prompts,
        "worker_lost_jobs": jobs.iter().filter(|status| status.status == "worker-lost").count(),
        "idle_timeout_jobs": jobs.iter().filter(|status| status.status == "idle-timeout").count(),
    }))
}

fn gateway_projects_json() -> Result<serde_json::Value> {
    let projects: Vec<_> = atelier_core::registry::list_projects()?
        .into_iter()
        .map(|project| serde_json::json!({"name": project.name, "path": project.path}))
        .collect();
    Ok(serde_json::json!({"projects": projects}))
}

fn gateway_jobs_json() -> Result<serde_json::Value> {
    let mut jobs = Vec::new();
    for project in atelier_core::registry::list_projects()? {
        for status in list_jobs(&project.path)? {
            jobs.push(serde_json::json!({
                "project": project.name,
                "id": status.id,
                "status": status.status,
                "thread_id": status.thread_id,
                "person": status.person,
            }));
        }
    }
    Ok(serde_json::json!({"jobs": jobs}))
}

fn gateway_prompts_json() -> Result<serde_json::Value> {
    let mut prompts = Vec::new();
    for project in atelier_core::registry::list_projects()? {
        for (job_id, prompt) in list_prompts(&project.path)? {
            if prompt.status == atelier_core::codex_app_server::PendingPromptStatus::Pending {
                prompts.push(serde_json::json!({
                    "project": project.name,
                    "job_id": job_id,
                    "id": prompt.id,
                    "summary": prompt.summary,
                    "method": prompt.method,
                    "available_decisions": prompt.available_decisions,
                }));
            }
        }
    }
    Ok(serde_json::json!({"prompts": prompts}))
}

fn respond_to_prompt(
    project_path: &Path,
    prompt_id: &str,
    decision: &str,
    text: Option<String>,
    json: Option<String>,
) -> Result<()> {
    let (job_dir, mut prompt) = find_prompt(project_path, prompt_id)?;
    let response = build_prompt_response(&prompt, decision, text, json)?;
    prompt.status = atelier_core::codex_app_server::PendingPromptStatus::Resolved;
    std::fs::write(
        job_dir.join("prompts").join(format!("{}.json", prompt.id)),
        serde_json::to_string_pretty(&prompt).context("serialize prompt")?,
    )?;
    let responses_dir = job_dir.join("responses");
    std::fs::create_dir_all(&responses_dir)?;
    std::fs::write(
        responses_dir.join(format!("{}.json", prompt.id)),
        serde_json::to_string_pretty(&response)?,
    )?;
    Ok(())
}

fn show_job(project_path: &Path, job_id: &str) -> Result<()> {
    let job_dir = project_path.join(".atelier/jobs").join(job_id);
    let status_path = job_dir.join("status.json");
    let mut status: atelier_core::job::JobStatus =
        serde_json::from_str(&std::fs::read_to_string(&status_path).context("read job status")?)
            .context("parse job status")?;
    reconcile_worker_status(&job_dir, &mut status)?;

    println!("Job: {}", status.id);
    println!("Status: {}", status.status);
    println!("Thread: {}", status.thread_id);
    println!("Person: {}", status.person);
    println!("Job directory: {}", job_dir.display());
    print_log_preview(&job_dir.join("worker-stdout.log"), "worker-stdout.log")?;
    print_log_preview(&job_dir.join("worker-stderr.log"), "worker-stderr.log")?;
    print_log_preview(&job_dir.join("stderr.log"), "stderr.log")?;
    Ok(())
}

fn print_log_preview(path: &Path, label: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path).with_context(|| format!("read {label}"))?;
    let first_line = content.lines().next().unwrap_or("");
    println!("{label}: {first_line}");
    Ok(())
}

fn ensure_project_writer_available(project_path: &Path) -> Result<()> {
    for status in list_jobs(project_path)? {
        if status.status == "running" || status.status == "waiting-for-prompt" {
            anyhow::bail!(
                "active managed job already owns project writer slot: {} ({})",
                status.id,
                status.status
            );
        }
    }
    Ok(())
}

fn list_jobs(project_path: &Path) -> Result<Vec<atelier_core::job::JobStatus>> {
    let jobs_dir = project_path.join(".atelier/jobs");
    let mut jobs = Vec::new();
    if !jobs_dir.exists() {
        return Ok(jobs);
    }
    for job_entry in std::fs::read_dir(jobs_dir).context("read jobs dir")? {
        let job_dir = job_entry?.path();
        let status_path = job_dir.join("status.json");
        if status_path.exists() {
            let mut status: atelier_core::job::JobStatus = serde_json::from_str(
                &std::fs::read_to_string(&status_path).context("read job status")?,
            )
            .context("parse job status")?;
            reconcile_worker_status(&job_dir, &mut status)?;
            jobs.push(status);
        }
    }
    jobs.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(jobs)
}

fn reconcile_worker_status(
    job_dir: &Path,
    status: &mut atelier_core::job::JobStatus,
) -> Result<()> {
    if status.status != "running" && status.status != "waiting-for-prompt" {
        return Ok(());
    }
    let worker_path = job_dir.join("worker.json");
    if !worker_path.exists() {
        return Ok(());
    }
    let worker: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&worker_path).context("read worker metadata")?,
    )
    .context("parse worker metadata")?;
    let Some(pid) = worker.get("pid").and_then(serde_json::Value::as_u64) else {
        return Ok(());
    };
    if !process_is_alive(pid) {
        status.status = "worker-lost".to_string();
        atelier_core::job::update_status(job_dir, status.clone())?;
    }
    Ok(())
}

fn process_is_alive(pid: u64) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/proc").join(pid.to_string()).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        true
    }
}

fn list_prompts(
    project_path: &Path,
) -> Result<Vec<(String, atelier_core::codex_app_server::PendingPrompt)>> {
    let jobs_dir = project_path.join(".atelier/jobs");
    let mut prompts = Vec::new();
    if !jobs_dir.exists() {
        return Ok(prompts);
    }
    for job_entry in std::fs::read_dir(jobs_dir).context("read jobs dir")? {
        let job_dir = job_entry?.path();
        let prompts_dir = job_dir.join("prompts");
        if !prompts_dir.exists() {
            continue;
        }
        let job_id = job_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown-job")
            .to_string();
        for prompt_entry in std::fs::read_dir(prompts_dir).context("read prompts dir")? {
            let prompt_path = prompt_entry?.path();
            if prompt_path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let prompt = serde_json::from_str(
                &std::fs::read_to_string(&prompt_path).context("read prompt file")?,
            )
            .context("parse prompt file")?;
            prompts.push((job_id.clone(), prompt));
        }
    }
    prompts.sort_by(|left, right| left.1.id.cmp(&right.1.id));
    Ok(prompts)
}

fn find_prompt(
    project_path: &Path,
    prompt_id: &str,
) -> Result<(PathBuf, atelier_core::codex_app_server::PendingPrompt)> {
    let jobs_dir = project_path.join(".atelier/jobs");
    for job_entry in std::fs::read_dir(jobs_dir).context("read jobs dir")? {
        let job_dir = job_entry?.path();
        let prompt_path = job_dir.join("prompts").join(format!("{prompt_id}.json"));
        if prompt_path.exists() {
            let prompt = serde_json::from_str(
                &std::fs::read_to_string(&prompt_path).context("read prompt file")?,
            )
            .context("parse prompt file")?;
            return Ok((job_dir, prompt));
        }
    }
    anyhow::bail!("prompt not found: {prompt_id}")
}

fn build_prompt_response(
    prompt: &atelier_core::codex_app_server::PendingPrompt,
    decision: &str,
    text: Option<String>,
    json: Option<String>,
) -> Result<serde_json::Value> {
    if let Some(json) = json {
        return serde_json::from_str(&json).context("parse prompt response JSON");
    }
    if let Some(text) = text {
        return Ok(serde_json::json!({"decision": decision, "text": text}));
    }
    if !prompt.available_decisions.is_empty()
        && !prompt
            .available_decisions
            .iter()
            .any(|available| available == decision)
    {
        anyhow::bail!(
            "decision '{decision}' is not available for {}; choose one of: {}",
            prompt.id,
            prompt.available_decisions.join(", ")
        );
    }
    Ok(serde_json::json!({"decision": decision}))
}

fn build_context(person: &str, thread: &str, prompt: &str) -> Result<String> {
    let person_memory = atelier_core::people::read_person_memory(person)?;
    let person_memory_section = if person_memory.trim().is_empty() {
        "Person memory: none\n".to_string()
    } else {
        format!("Person memory:\n{}\n", person_memory.trim())
    };
    Ok(format!(
        "<atelier-context>\nCurrent person: {person}\nThread: {thread}\n{person_memory_section}Boundary:\n- This context is about the current person and invocation.\n- Person memory must only describe the person, never project facts.\n- Project facts belong in project files.\n</atelier-context>\n\n<user-task>\n{prompt}\n</user-task>\n"
    ))
}

fn run_managed_app_server_job(
    job: &atelier_core::job::CreatedJob,
    project_path: &Path,
    thread: &str,
    person: &str,
    context: &str,
    idle_timeout_seconds: u64,
) -> Result<()> {
    let exe = std::env::current_exe().context("current atelier executable")?;
    let child = ProcessCommand::new(exe)
        .arg("__managed-worker")
        .arg("--job-dir")
        .arg(&job.dir)
        .arg("--project-path")
        .arg(project_path)
        .arg("--thread")
        .arg(thread)
        .arg("--as")
        .arg(person)
        .arg("--idle-timeout-seconds")
        .arg(idle_timeout_seconds.to_string())
        .arg(context)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            std::fs::File::create(job.dir.join("worker-stdout.log"))
                .context("create worker stdout log")?,
        ))
        .stderr(Stdio::from(
            std::fs::File::create(job.dir.join("worker-stderr.log"))
                .context("create worker stderr log")?,
        ))
        .spawn()
        .context("spawn managed worker")?;

    std::fs::write(
        job.dir.join("worker.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "pid": child.id(),
            "idle_timeout_seconds": idle_timeout_seconds
        }))?,
    )?;

    let prompt_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(prompt) = first_prompt(&job.dir)? {
            println!("Job: {}", job.id);
            println!("Job directory: {}", job.dir.display());
            println!("Pending prompt: {}", prompt.id);
            println!("{}", prompt.summary);
            return Ok(());
        }
        let status_path = job.dir.join("status.json");
        let status_text = std::fs::read_to_string(&status_path).unwrap_or_default();
        if status_text.contains("\"status\": \"succeeded\"") {
            println!("Job: {}", job.id);
            println!("Job directory: {}", job.dir.display());
            println!("Status: succeeded");
            return Ok(());
        }
        if Instant::now() > prompt_deadline {
            println!("Job: {}", job.id);
            println!("Job directory: {}", job.dir.display());
            println!("Status: running");
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn first_prompt(job_dir: &Path) -> Result<Option<atelier_core::codex_app_server::PendingPrompt>> {
    let prompts_dir = job_dir.join("prompts");
    if !prompts_dir.exists() {
        return Ok(None);
    }
    let mut paths: Vec<_> = std::fs::read_dir(prompts_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    paths.sort();
    let Some(path) = paths.into_iter().next() else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&std::fs::read_to_string(path)?)?))
}

fn run_managed_worker(
    job_dir: &Path,
    project_path: &Path,
    thread: &str,
    person: &str,
    context: &str,
    idle_timeout_seconds: u64,
) -> Result<()> {
    let mut child = ProcessCommand::new("codex")
        .arg("app-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("start codex app-server")?;

    let mut stdin = child.stdin.take().context("codex app-server stdin")?;
    let stdout = child.stdout.take().context("codex app-server stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut protocol = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(job_dir.join("protocol.jsonl"))
        .context("open protocol log")?;

    send_json(
        &mut stdin,
        serde_json::json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"atelier","title":"Atelier","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true,"requestAttestation":false}}}),
    )?;
    send_json(
        &mut stdin,
        serde_json::json!({"method":"initialized","params":{}}),
    )?;
    send_json(
        &mut stdin,
        serde_json::json!({"id":2,"method":"thread/start","params":{"cwd":project_path,"approvalPolicy":"on-request","sandbox":"workspace-write"}}),
    )?;
    let thread_start = read_until_response_thread(&mut reader, &mut protocol, 2)?;
    let codex_thread_id = thread_start
        .get("id")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .context("thread/start response missing thread id")?;
    atelier_core::thread::append_codex_session_lineage(
        project_path,
        thread,
        serde_json::json!({
            "kind": "managed-app-server-thread",
            "job_id": job_dir.file_name().and_then(|value| value.to_str()).unwrap_or("unknown-job"),
            "codex_thread_id": codex_thread_id,
            "session_path": thread_start.get("path").and_then(serde_json::Value::as_str),
        }),
    )?;
    send_json(
        &mut stdin,
        serde_json::json!({"id":3,"method":"turn/start","params":{"threadId":codex_thread_id,"input":[{"type":"text","text":context,"textElements":[]}]}}),
    )?;

    write_job_status(job_dir, "running", thread, person)?;
    let idle_deadline = Instant::now() + Duration::from_secs(idle_timeout_seconds);
    let mut line = String::new();
    loop {
        if Instant::now() > idle_deadline {
            write_job_status(job_dir, "idle-timeout", thread, person)?;
            let _ = child.kill();
            return Ok(());
        }
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .context("read codex app-server")?;
        if bytes == 0 {
            write_job_status(job_dir, "failed", thread, person)?;
            return Ok(());
        }
        protocol
            .write_all(line.as_bytes())
            .context("write protocol")?;
        let trimmed = line.trim_end();
        if let Some(prompt) = atelier_core::codex_app_server::parse_pending_prompt(trimmed) {
            persist_prompt(job_dir, &prompt)?;
            write_job_status(job_dir, "waiting-for-prompt", thread, person)?;
            wait_for_prompt_response(job_dir, &prompt, &mut stdin, idle_timeout_seconds)?;
            write_job_status(job_dir, "running", thread, person)?;
            continue;
        }
        if let Some(message) = agent_message_text(trimmed) {
            std::fs::write(job_dir.join("result.md"), message)?;
        }
        if message_method(trimmed).as_deref() == Some("turn/completed") {
            write_job_status(job_dir, "succeeded", thread, person)?;
            return Ok(());
        }
    }
}

fn persist_prompt(
    job_dir: &Path,
    prompt: &atelier_core::codex_app_server::PendingPrompt,
) -> Result<()> {
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).context("create prompts dir")?;
    std::fs::write(
        prompts_dir.join(format!("{}.json", prompt.id)),
        serde_json::to_string_pretty(prompt).context("serialize prompt")?,
    )
    .context("write prompt")
}

fn wait_for_prompt_response(
    job_dir: &Path,
    prompt: &atelier_core::codex_app_server::PendingPrompt,
    stdin: &mut std::process::ChildStdin,
    idle_timeout_seconds: u64,
) -> Result<()> {
    let response_path = job_dir
        .join("responses")
        .join(format!("{}.json", prompt.id));
    let deadline = Instant::now() + Duration::from_secs(idle_timeout_seconds);
    loop {
        if response_path.exists() {
            let response: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&response_path).context("read prompt response")?,
            )
            .context("parse prompt response")?;
            send_json(
                stdin,
                serde_json::json!({"id": prompt.codex_request_id.parse::<i64>().unwrap_or(0), "result": response}),
            )?;
            return Ok(());
        }
        if Instant::now() > deadline {
            anyhow::bail!("prompt response timed out: {}", prompt.id);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn agent_message_text(line: &str) -> Option<String> {
    let message: serde_json::Value = serde_json::from_str(line).ok()?;
    let item = message.get("params")?.get("item")?;
    if item.get("type")?.as_str()? == "agentMessage" {
        item.get("text")?.as_str().map(ToString::to_string)
    } else {
        None
    }
}

fn message_method(line: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()?
        .get("method")?
        .as_str()
        .map(ToString::to_string)
}

fn write_job_status(job_dir: &Path, status: &str, thread: &str, person: &str) -> Result<()> {
    let id = job_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown-job")
        .to_string();
    atelier_core::job::update_status(
        job_dir,
        atelier_core::job::JobStatus {
            id,
            status: status.to_string(),
            thread_id: thread.to_string(),
            person: person.to_string(),
            dry_run: false,
            exit_code: None,
            codex_binary: Some("codex".to_string()),
            invocation: vec!["app-server".to_string()],
        },
    )
}

fn send_json(stdin: &mut std::process::ChildStdin, value: serde_json::Value) -> Result<()> {
    writeln!(stdin, "{}", serde_json::to_string(&value)?).context("write app-server message")?;
    stdin.flush().context("flush app-server message")
}

fn read_until_response_thread(
    reader: &mut BufReader<std::process::ChildStdout>,
    protocol: &mut std::fs::File,
    response_id: i64,
) -> Result<serde_json::Value> {
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .context("read app-server response")?;
        if bytes == 0 {
            anyhow::bail!("codex app-server closed before thread/start response");
        }
        protocol
            .write_all(line.as_bytes())
            .context("write protocol")?;
        let message: serde_json::Value = serde_json::from_str(line.trim_end())
            .ok()
            .unwrap_or_default();
        if message.get("id").and_then(serde_json::Value::as_i64) == Some(response_id) {
            return message
                .get("result")
                .and_then(|result| result.get("thread"))
                .cloned()
                .context("thread/start response missing thread metadata");
        }
    }
}

fn finish_job(
    job: &atelier_core::job::CreatedJob,
    thread: &str,
    person: &str,
    output: atelier_core::codex::CodexRunOutput,
) -> Result<()> {
    std::fs::write(job.dir.join("result.md"), &output.stdout)?;
    if output.stdout.is_empty() {
        std::fs::write(
            job.dir.join("interactive-output.md"),
            "Interactive job output was streamed directly to the attached terminal.\n",
        )?;
    }
    std::fs::write(job.dir.join("stderr.log"), &output.stderr)?;
    atelier_core::job::complete_job(job, thread, person, &output)?;
    println!("Job: {}", job.id);
    println!("Job directory: {}", job.dir.display());
    println!(
        "Status: {}",
        if output.success {
            "succeeded"
        } else {
            "failed"
        }
    );
    print!("{}", output.stdout);
    if !output.success {
        eprint!("{}", output.stderr);
        std::process::exit(output.exit_code.unwrap_or(1));
    }
    Ok(())
}
