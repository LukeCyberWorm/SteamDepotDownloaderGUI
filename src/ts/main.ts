import $ from "jquery";
import {invoke} from "@tauri-apps/api/core";
import {open as abrirDialogo} from "@tauri-apps/plugin-dialog";
import {open as abrirShell} from "@tauri-apps/plugin-shell";
import {listen as ouvir} from "@tauri-apps/api/event";

function definirCarregador(estado: boolean) {
    $("#busy").prop("hidden", !estado);
}

function definirEstadoCarregamento(estado: boolean) {
    $("#busy").prop("hidden", !estado);

    // Iterar por todos os botões e campos de entrada e desativá-los
    for (const elemento of document.querySelectorAll("button, input")) {
        if (elemento.closest("#settings-content")) continue;
        (elemento as any).disabled = estado;
    }

    // Esses elementos precisam de propriedades adicionais para serem desativados corretamente
    $("#pickpath").prop("ariaDisabled", estado);
    $("#downloadbtn").prop("ariaDisabled", estado);

    // Desativar botões da internet
    for (const elemento of document.querySelectorAll("#internet-btns div")) {
        elemento.ariaDisabled = String(estado);
    }
}

/// Retorna uma lista de IDs de campos de formulário inválidos
const camposInvalidos = () => {
    const formulario = document.forms[0];

    const camposInvalidos: string[] = [];
    for (const entrada of formulario) {
        const elementoEntrada = entrada as HTMLInputElement;
        const valido = !(elementoEntrada.value === "" && elementoEntrada?.parentElement?.classList.contains("required"));
        if (!valido) {
            camposInvalidos.push(elementoEntrada.id);
        }
    }
    // console.debug(`[${camposInvalidos.join(", ")}] campos inválidos/vazios`);

    return camposInvalidos;
};

$(async () => {
    let terminaisColetados = false;
    let diretorioDownload: string | null;

    // Lógica de inicialização
    definirEstadoCarregamento(true);

    await invoke("preload_vectum");

    definirEstadoCarregamento(false);

    // Coletar o restante dos terminais em segundo plano
    if (!terminaisColetados) {
        definirCarregador(true);
        // @ts-ignore
        const terminais = await invoke("get_all_terminals") as string[];
        for (const terminal in terminais) {
            console.log(terminal);
        }

        // Permitir abrir configurações agora que está pronto para ser exibido
        $("#settings-button").prop("ariaDisabled", false);
        terminaisColetados = true;
        definirCarregador(false);
    }

    $("#pickpath").on("click", async () => {
        // Abrir um diálogo
        diretorioDownload = await abrirDialogo({
            title: "Escolha onde salvar o download do jogo.",
            multiple: false,
            directory: true,
            canCreateDirectories: true
        });

        if (diretorioDownload == null) {
            // usuário cancelou
            $("#checkpath").prop("ariaDisabled", true);
            $("#checkpath").prop("disabled", true);
            return;
        }

        $("#checkpath").prop("ariaDisabled", false);
        $("#checkpath").prop("disabled", false);
        $("#downloadbtn").prop("ariaDisabled", false);
        $("#nopathwarning").prop("hidden", true);

        console.log(diretorioDownload);
    });

    $("#checkpath").on("click", async () => {
        console.log(`Verificando caminho: ${diretorioDownload}`);

        if (diretorioDownload != null) {
            await abrirShell(diretorioDownload);
        } else {
            $("#checkpath").prop("ariaDisabled", true);
        }
    });

    $("#downloadbtn").on("click", async () => {
        console.log("Botão de download clicado");

        if (camposInvalidos().length > 0) {
            // Iterar pelos campos inválidos. Se houver algum, marcar como "errored" e bloquear o botão de download
            for (const id of camposInvalidos()) {
                document.getElementById(id)?.parentElement?.classList.toggle("errored", true);
            }
            $("#emptywarning").prop("hidden", false);
            $("#downloadbtn").prop("ariaDisabled", true);
            return;
        }

        if (diretorioDownload == null) {
            $("#nopathwarning").prop("hidden", false);
            $("#downloadbtn").prop("ariaDisabled", true);
            return;
        }

        definirEstadoCarregamento(true);
        $("#downloadingnotice").prop("hidden", false);
        $("#busy").prop("hidden", true); // Não mostrar o carregador desta vez

        const escolhaTerminal = (document.getElementById("terminal-dropdown") as HTMLSelectElement).selectedIndex;
        const escolhaNomeDiretorio = $("#folder-name-custom-input").val();

        // Caminho de saída com os diretórios escolhidos é: {diretorioDownload}/{escolhaNomeDiretorio}
        const opcoesVectum = {
            terminal: escolhaTerminal == 13 ? null : escolhaTerminal,
            diretorio_saida: diretorioDownload || null, // se não especificado, deixe o backend escolher um caminho
            nome_diretorio: escolhaNomeDiretorio || null,
        };

        const downloadSteam = {
            // String || null traduz para Some(String) || None
            usuario: String($("#username").val()).trim() || null,
            senha: String($("#password").val()).trim() || null,
            app_id: $("#appid").val(),
            depot_id: $("#depotid").val(),
            manifest_id: $("#manifestid").val(),
            opcoes: opcoesVectum
        };

        // console.debug(downloadSteam);
        await invoke("download_depotdownloader");

        $("#downloadingnotice").prop("hidden", true);
        definirEstadoCarregamento(false);

        console.debug("Processo de download do DepotDownloader concluído. Iniciando download do jogo...");

        await invoke("start_download", {steamDownload: downloadSteam});
        console.log("Dados do frontend enviados para o backend. Pronto para o próximo download.");
    });

    $("#settings-button").on("click", async () => {
        if (terminaisColetados) $("#settings-surrounding").css("display", "block");
    });

    $("#settings-surrounding").on("click", (evento) => {
        if (evento.target === document.getElementById("settings-surrounding")) {
            $("#settings-surrounding").css("display", "none");
        }
    });

    $("#opium-btn").on("click", () => {
        abrirShell("https://aphex.cc");
    });

    document.forms[0].addEventListener("input", (evento) => {
        // Remover classe "errored". Este é um jeito ruim de fazer, mas funciona por enquanto
        const alvo = evento.target as HTMLElement;
        alvo?.parentElement?.classList.toggle("errored", false);

        // Se não houver mais campos inválidos, esconder o aviso e habilitar o botão de download novamente
        if (camposInvalidos().length === 0) {
            $("#emptywarning").prop("hidden", true);
            $("#downloadbtn").prop("ariaDisabled", false);
        }
    });
});

let a = 0;
// Cada terminal instalado é recebido do Rust com este evento
ouvir<[number, number]>("working-terminal", (evento) => {
    a++;
    console.log(
        `Terminal #${evento.payload[0]} está instalado. a = ${a}`
    );
    const selecaoTerminal = (document.getElementById("terminal-dropdown") as HTMLSelectElement);

    // Habilitar a <option> do terminal porque sabemos que está disponível. Ignorar verificação nula porque sabemos que é válido
    // @ts-ignore
    selecaoTerminal.options.item(evento.payload[0]).disabled = false;
    // @ts-ignore 16

    selecaoTerminal.options.item(evento.payload[0]).text = selecaoTerminal.options.item(evento.payload[0]).text.slice(0, -16);

    $("#terminals-found").text(`${a}/${evento.payload[1]}`);
});

ouvir<string>("default-terminal", (evento) => {
    console.log(
        `Terminal padrão é ${evento.payload}.`
    );

    $("#default-terminal").text(evento.payload);
});