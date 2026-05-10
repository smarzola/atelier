use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

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
        /// Use Codex app-server managed mode for structured prompt relay.
        #[arg(long)]
        managed: bool,
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
enum GatewayCommand {
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
        /// Decision to record.
        decision: String,
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
                println!(
                    "Initialized Atelier project '{}' at {}",
                    name,
                    path.display()
                );
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
        Command::Gateway { command } => match command {
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
            PromptsCommand::List { project_path } => {
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
                decision,
            } => {
                let (job_dir, mut prompt) = find_prompt(&project_path, &prompt_id)?;
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
                    serde_json::to_string_pretty(&serde_json::json!({"decision": decision}))?,
                )?;
                println!("Recorded response {decision} for {prompt_id}");
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
        Command::Work {
            project_path,
            thread,
            person,
            dry_run,
            interactive,
            managed,
            approval_policy,
            sandbox,
            model,
            search,
            prompt,
        } => {
            let policy = atelier_core::codex::CodexPolicy {
                approval_policy,
                sandbox,
                model,
                search,
            };
            let context = build_context(&person, &thread, &prompt)?;
            let job = atelier_core::job::create_job(
                &project_path,
                &thread,
                &person,
                &prompt,
                &context,
                dry_run,
            )?;
            let invocation = atelier_core::codex::CodexInvocation::with_policy(
                &project_path,
                context.clone(),
                policy,
            );

            if dry_run {
                println!("Job: {}", job.id);
                println!("Job directory: {}", job.dir.display());
                println!("Would run: {}", invocation.display_command());
                println!("\n{}", invocation.prompt);
            } else if managed {
                run_managed_app_server_job(&job, &project_path, &thread, &person, &context)?;
            } else if interactive {
                let output = invocation.run_interactive()?;
                finish_job(&job, &thread, &person, output)?;
            } else {
                let output = invocation.run()?;
                finish_job(&job, &thread, &person, output)?;
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
            print!(
                "{}",
                atelier_core::thread::codex_session_lineage(&project_path, &thread)?
            );
        }
    }

    Ok(())
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
    let protocol_path = job.dir.join("protocol.jsonl");
    let mut protocol = std::fs::File::create(&protocol_path).context("create protocol log")?;

    send_json(
        &mut stdin,
        serde_json::json!({
            "id": 1,
            "method": "initialize",
            "params": {
                "clientInfo": {"name": "atelier", "title": "Atelier", "version": env!("CARGO_PKG_VERSION")},
                "capabilities": {"experimentalApi": true, "requestAttestation": false}
            }
        }),
    )?;
    send_json(
        &mut stdin,
        serde_json::json!({"method": "initialized", "params": {}}),
    )?;
    send_json(
        &mut stdin,
        serde_json::json!({
            "id": 2,
            "method": "thread/start",
            "params": {"cwd": project_path, "approvalPolicy": "on-request", "sandbox": "workspace-write"}
        }),
    )?;

    let codex_thread_id = read_until_response_thread(&mut reader, &mut protocol, 2)?;
    send_json(
        &mut stdin,
        serde_json::json!({
            "id": 3,
            "method": "turn/start",
            "params": {
                "threadId": codex_thread_id,
                "input": [{"type": "text", "text": context, "textElements": []}]
            }
        }),
    )?;

    let mut waiting_for_prompt = false;
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .context("read codex app-server")?;
        if bytes == 0 {
            break;
        }
        protocol
            .write_all(line.as_bytes())
            .context("write protocol")?;
        if let Some(prompt) = atelier_core::codex_app_server::parse_pending_prompt(line.trim_end())
        {
            let prompts_dir = job.dir.join("prompts");
            std::fs::create_dir_all(&prompts_dir).context("create prompts dir")?;
            std::fs::write(
                prompts_dir.join(format!("{}.json", prompt.id)),
                serde_json::to_string_pretty(&prompt).context("serialize prompt")?,
            )
            .context("write prompt")?;
            println!("Job: {}", job.id);
            println!("Job directory: {}", job.dir.display());
            println!("Pending prompt: {}", prompt.id);
            println!("{}", prompt.summary);
            waiting_for_prompt = true;
            break;
        }
    }

    let status = atelier_core::job::JobStatus {
        id: job.id.clone(),
        status: if waiting_for_prompt {
            "waiting-for-prompt"
        } else {
            "succeeded"
        }
        .to_string(),
        thread_id: thread.to_string(),
        person: person.to_string(),
        dry_run: false,
        exit_code: None,
        codex_binary: Some("codex".to_string()),
        invocation: vec!["app-server".to_string()],
    };
    atelier_core::job::update_status(&job.dir, status)?;
    let _ = child.kill();
    Ok(())
}

fn send_json(stdin: &mut std::process::ChildStdin, value: serde_json::Value) -> Result<()> {
    writeln!(stdin, "{}", serde_json::to_string(&value)?).context("write app-server message")?;
    stdin.flush().context("flush app-server message")
}

fn read_until_response_thread(
    reader: &mut BufReader<std::process::ChildStdout>,
    protocol: &mut std::fs::File,
    response_id: i64,
) -> Result<String> {
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
            return message["result"]["thread"]["id"]
                .as_str()
                .map(ToString::to_string)
                .context("thread/start response missing thread id");
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
