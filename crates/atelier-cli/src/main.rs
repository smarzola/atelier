use std::path::PathBuf;

use anyhow::Result;
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
            prompt,
        } => {
            let context = build_context(&person, &thread, &prompt)?;
            let job = atelier_core::job::create_job(
                &project_path,
                &thread,
                &person,
                &prompt,
                &context,
                dry_run,
            )?;
            let invocation =
                atelier_core::codex::CodexInvocation::new(&project_path, context.clone());

            if dry_run {
                println!("Job: {}", job.id);
                println!("Job directory: {}", job.dir.display());
                println!("Would run: {}", invocation.display_command());
                println!("\n{}", invocation.prompt);
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
            prompt,
        } => {
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
                atelier_core::codex::CodexResumeInvocation::last(context)
            } else if let Some(session) = session {
                atelier_core::codex::CodexResumeInvocation::session(session, context)
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

fn finish_job(
    job: &atelier_core::job::CreatedJob,
    thread: &str,
    person: &str,
    output: atelier_core::codex::CodexRunOutput,
) -> Result<()> {
    std::fs::write(job.dir.join("result.md"), &output.stdout)?;
    std::fs::write(job.dir.join("stderr.log"), &output.stderr)?;
    atelier_core::job::complete_job(job, thread, person, output.success)?;
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
        std::process::exit(1);
    }
    Ok(())
}
