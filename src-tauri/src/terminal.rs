use crate::get_os;
use crate::steam::SteamDownload;
use std::fs;
use std::path::PathBuf;
use tauri::Wry;
use tauri_plugin_shell::process::Command;
use tauri_plugin_shell::Shell;

/// Representa um terminal que pode ser usado para executar comandos.
/// **Deve estar sincronizado com o menu suspenso do terminal no frontend.**
#[derive(Debug, PartialEq)]
pub enum Terminal {
    GNOMETerminal,
    Alacritty,
    Konsole,
    GNOMEConsole,
    Xfce4Terminal,
    DeepinTerminal,
    Terminator,
    Kitty,
    LXTerminal,
    Tilix,
    XTerm,
    CMD,
    Terminal
}


impl Terminal {
    /// Itera por cada terminal
    pub fn iter() -> impl Iterator<Item=Terminal> {
        use self::Terminal::*;

        vec![
            GNOMETerminal, Alacritty, Konsole, GNOMEConsole, Xfce4Terminal, DeepinTerminal, Terminator, Kitty, LXTerminal, Tilix, XTerm, CMD, Terminal
        ].into_iter()
    }

    /// Obter terminal do índice na ordem do [`Terminal`] enum
    pub fn from_index(index: &u8) -> Option<Terminal> {
        Terminal::iter().nth(*index as usize)
    }

    /// Obtenha o índice de um terminal na ordem do [`Terminal`] enum
    /// Retorna `None` se o terminal não for encontrado.
    pub fn index(&self) -> Option<u8> {
        Terminal::iter().position(|x| x == *self).map(|x| x as u8)
    }


    /// Obtenha o número total de terminais **possíveis** dependendo do sistema operacional
    pub fn total() -> u8 {
        if get_os() == "windows" || get_os() == "macos" {
            return 1;
        }

        Terminal::iter().count() as u8 - 1 // -1 porque cmd não está disponível no linux
    }

    /// Get the pretty name of a terminal
    pub fn pretty_name(&self) -> &str {
        match self {
            Terminal::GNOMETerminal => "GNOME Terminal",
            Terminal::GNOMEConsole => "GNOME Console",
            Terminal::Konsole => "Konsole",
            Terminal::Xfce4Terminal => "Xfce Terminal",
            Terminal::Terminator => "Terminator",
            Terminal::XTerm => "XTerm",
            Terminal::Kitty => "Kitty",
            Terminal::LXTerminal => "LXTerminal",
            Terminal::Tilix => "Tilix",
            Terminal::DeepinTerminal => "Deepin Terminal",
            Terminal::Alacritty => "Alacritty",
            Terminal::CMD => "cmd",
            Terminal::Terminal => "Terminal"
        }
    }


    //região onde é sondado um terminal
    /// Verifica se um [`Terminal`] está instalado.
    /// **Ver:** [`get_installed_terminals`]
    pub async fn installed(&self, shell: &Shell<Wry>) -> bool {
        match self {
            Terminal::CMD => get_os() == "windows",
            Terminal::GNOMETerminal => shell.command("gnome-terminal").arg("--version").status().await.is_ok(),
            Terminal::GNOMEConsole => shell.command("kgx").arg("--version").status().await.is_ok(),
            Terminal::Konsole => shell.command("konsole").arg("--version").status().await.is_ok(),
            Terminal::Xfce4Terminal => shell.command("xfce4-terminal").arg("--version").status().await.is_ok(),
            Terminal::Terminator => shell.command("terminator").arg("--version").status().await.is_ok(),
            Terminal::XTerm => shell.command("xterm").arg("-v").status().await.is_ok(),
            Terminal::Kitty => shell.command("kitty").arg("--version").status().await.is_ok(),
            Terminal::LXTerminal => shell.command("lxterminal").arg("--version").status().await.is_ok(),
            Terminal::Tilix => shell.command("tilix").arg("--version").status().await.is_ok(),
            Terminal::DeepinTerminal => shell.command("deepin-terminal").arg("--version").status().await.is_ok(),
            Terminal::Alacritty => shell.command("alacritty").arg("--version").status().await.is_ok(),
            Terminal::Terminal => get_os() == "macos",
        }
    }
    //endregion


    //region Running a command in the terminal
    /**
    Returns a [`Command`] that, when executed should open the terminal and run the command.


    ## Commands
    `{command}` = `{command};echo Command finished with code $?;sleep infinity`

    | Terminal         | Command to open terminal                                                 |
    |------------------|--------------------------------------------------------------------------|
    | cmd              | `start cmd.exe /k  {command}`                                            |
    | GNOMETerminal    | `gnome-terminal -- /usr/bin/env sh -c  {command}`                        |
    | GNOMEConsole     | `kgx -e /usr/bin/env sh -c  {command}`                                   |
    | Konsole          | `konsole -e /usr/bin/env sh -c  {command}`                               |
    | Xfce4Terminal    | `xfce4-terminal -x /usr/bin/env sh -c  {command}`                        |
    | Terminator       | `terminator -T "Downloading depot..." -e  {command}`                     |
    | XTerm            | `xterm -hold -T "Downloading depot..." -e /usr/bin/env sh -c  {command}` |
    | Kitty            | `kitty /usr/bin/env sh -c  {command}`                                    |
    | LXTerminal       | `lxterminal -e /usr/bin/env sh -c  {command}`                            |
    | Tilix            | `tilix -e /usr/bin/env sh -c  {command}`                                 |
    | DeepinTerminal   | `deepin-terminal -e /usr/bin/env sh -c  {command}`                       |
    | Alacritty        | `alacritty -e /usr/bin/env sh -c  {command}`                             |
    | Terminal (macOS) | We create a bash script and run that using `open`.                       |

     */
    pub fn create_command(&self, steam_download: &SteamDownload, shell: &Shell<Wry>, working_dir: &PathBuf) -> Command {
        let command = create_depotdownloader_command(steam_download);

        match self {
            Terminal::CMD => {
                    return shell.command("cmd.exe").args(&["/c", "start", "PowerShell.exe", "-NoExit", "-Command"]).args(command);

/*                let mut cmd = std::process::Command::new("cmd.exe");
                cmd.args(&["/c", "start", "PowerShell.exe", "-NoExit", "-Command"]).args(command);

                return cmd*/
            }
            Terminal::GNOMETerminal => {
                shell.command("gnome-terminal")
                    .args(&["--", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::GNOMEConsole => {
                shell.command("kgx")
                    .args(&["-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::Konsole => {
                shell.command("konsole")
                    .args(&["-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::Xfce4Terminal => {
                shell.command("xfce4-terminal")
                    .args(&["-x", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::Terminator => {
                shell.command("terminator")
                    .args(&["-T", "Downloading depot...", "-e"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::XTerm => {
                shell.command("xterm")
                    .args(&["-hold", "-T", "Downloading depot...", "-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::Kitty => {
                shell.command("kitty")
                    .args(&["/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::LXTerminal => {
                shell.command("lxterminal")
                    .args(&["-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::Tilix => {
                shell.command("tilix")
                    .args(&["-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::DeepinTerminal => {
                shell.command("deepin-terminal")
                    .args(&["-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }

            Terminal::Alacritty => {
                shell.command("alacritty")
                    .args(&["-e", "/usr/bin/env", "sh", "-c"])
                    .args(command)
                    .current_dir(working_dir.as_path())
            }
            Terminal::Terminal => {
                // Crie um script bash e executar. Não é muito seguro, mas torna isso mais fácil.
                let download_script = format!("#!/bin/bash\ncd {}\n{}",working_dir.to_str().unwrap().replace(" ", "\\ "), command[0]);

                fs::write("./script.sh", download_script).unwrap();

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions("./script.sh", fs::Permissions::from_mode(0o755)).unwrap(); // Não será executado sem permissão executável
                }

                shell.command("/usr/bin/open")
                    .args(&["-a", "Terminal", "./script.sh"])
                    .current_dir(working_dir.as_path())

            }
        }
    }
    //endregion
}

/**
Verifica se os terminais estão instalados verificando se eles respondem aos comandos.

## Como funciona
Investiga uma lista de terminais populares e verifica se eles retornam um erro ao chamar seu `--version` ou sinalizador de linha de comando similar.

## Options
* `return_immediately`: [`bool`]: Retorna assim que um terminal for encontrado.

## Returns
Um vetor contendo uma lista de terminais que devem funcionar.

## Commands
| Terminal       | Command to check if installed |
|----------------|-------------------------------|
| cmd            | `cmd /?`                      |
| GNOMETerminal  | `gnome-terminal --version`    |
| GNOMEConsole   | `kgx --version`               |
| Konsole        | `konsole --version`           |
| Xfce4Terminal  | `xfce4-terminal --version`    |
| Terminator     | `terminator --version`        |
| XTerm          | `xterm -v`                    |
| Kitty          | `kitty --version`             |
| LXTerminal     | `lxterminal --version`        |
| Tilix          | `tilix --version`             |
| DeepinTerminal | `deepin-terminal --version`   |
| Alacritty      | `alacritty --version`         |

 */
pub async fn get_installed_terminals(return_immediately: bool, shell: &Shell<Wry>) -> Vec<Terminal> {
    match get_os() {
        "windows" => { return vec!(Terminal::CMD); }
        "macos" => { return vec!(Terminal::Terminal); }
        _ => {}
    }


    let mut available_terminals: Vec<Terminal> = Vec::new();

    for terminal in Terminal::iter() {
        // Terminal de sondagem. Se não gerar erro, provavelmente está instalado.
        if terminal.installed(shell).await {
            if return_immediately {
                return vec![terminal];
            }
            available_terminals.push(terminal);
        }
    }

    if available_terminals.is_empty() {
        eprintln!("No terminals were detected. Try installing one.");
    }

    available_terminals
}

/// Cria o comando DepotDownloader necessário para baixar o manifest solicitado.
fn create_depotdownloader_command(steam_download: &SteamDownload) -> Vec<String> {
    let output_dir = if get_os() == "windows" {
        // No PowerShell, os espaços podem ser escapados com uma crase.
        steam_download.output_path().replace(" ", "` ")
    } else {
        // No bash, os espaços podem ser escapados com uma barra invertida.
        steam_download.output_path().replace(" ", "\\ ")
    };


    if cfg!(not(windows)) {
        if steam_download.is_anonymous() {
            vec![format!(r#"./DepotDownloader -app {} -depot {} -manifest {} -dir {};echo Done!;sleep infinity"#, steam_download.app_id(), steam_download.depot_id(), steam_download.manifest_id(), output_dir)]
        } else {
            vec![format!(r#"./DepotDownloader -username {} -password {} -app {} -depot {} -manifest {} -dir {};echo Done!;sleep infinity"#, steam_download.username().clone().unwrap(), steam_download.password().clone().unwrap(), steam_download.app_id(), steam_download.depot_id(), steam_download.manifest_id(), output_dir)]
        }
    } else {
        if steam_download.is_anonymous() {
            vec![format!(r#".\DepotDownloader.exe -app {} -depot {} -manifest {} -dir {}"#, steam_download.app_id(), steam_download.depot_id(), steam_download.manifest_id(), output_dir)]
        } else {
            vec![format!(r#".\DepotDownloader.exe -username {} -password {} -app {} -depot {} -manifest {} -dir {}"#, steam_download.username().clone().unwrap(), steam_download.password().clone().unwrap(), steam_download.app_id(), steam_download.depot_id(), steam_download.manifest_id(), output_dir)]
        }
    }
}
