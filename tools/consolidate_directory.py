#!/usr/bin/env python3
"""
Script para consolidar estrutura de diretório em um único arquivo de texto
Lê todos os arquivos de um diretório e subdiretórios e consolida em formato:
/caminho/arquivo: Conteúdo completo do arquivo...

Uso: python consolidate_directory.py <diretorio_origem> <arquivo_destino>
"""

import os
import sys
import argparse
from pathlib import Path

def read_directory_structure(directory_path):
    """
    Lê a estrutura de um diretório e retorna todos os arquivos com seu conteúdo
    no formato especificado: /caminho/arquivo: Conteúdo completo do arquivo...
    """
    directory = Path(directory_path)
    if not directory.exists():
        raise FileNotFoundError(f"Diretório não encontrado: {directory_path}")

    if not directory.is_dir():
        raise ValueError(f"O caminho especificado não é um diretório: {directory_path}")

    result_content = []

    # Lista para armazenar todos os arquivos encontrados
    files_found = []

    # Percorre todos os arquivos no diretório e subdiretórios
    for file_path in sorted(directory.rglob("*")):
        if file_path.is_file():
            files_found.append(file_path)

    print(f"Encontrados {len(files_found)} arquivos")

    for file_path in files_found:
        # Calcula o caminho relativo a partir do diretório raiz
        relative_path = file_path.relative_to(directory)

        try:
            # Tenta ler o arquivo como texto
            with open(file_path, 'r', encoding='utf-8') as file:
                content = file.read()

            # Adiciona no formato especificado
            result_content.append(f"{relative_path}:\n\n{content}")
            print(f"Processado: {relative_path}")

        except UnicodeDecodeError:
            # Se não conseguir ler como texto, pula o arquivo
            print(f"Aviso: Pulando arquivo binário ou com encoding incompatível: {relative_path}")
            continue
        except Exception as e:
            print(f"Erro ao ler arquivo {relative_path}: {e}")
            continue

    return "\n\n".join(result_content)

def write_to_file(content, output_path):
    """
    Escreve o conteúdo no arquivo de saída
    """
    output_file = Path(output_path)

    # Cria o diretório se não existir
    output_file.parent.mkdir(parents=True, exist_ok=True)

    try:
        with open(output_file, 'w', encoding='utf-8') as file:
            file.write(content)
        print(f"Arquivo salvo com sucesso: {output_path}")
    except Exception as e:
        raise Exception(f"Erro ao escrever arquivo: {e}")

def main():
    parser = argparse.ArgumentParser(
        description="Lê a estrutura de um diretório e consolida todos os arquivos em um único arquivo de texto",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Exemplos de uso:
  python consolidate_directory.py /path/to/source/directory /path/to/output/file.txt
  python consolidate_directory.py ./meu_projeto ./consolidated_code.txt
  python consolidate_directory.py ../src ../output/all_files.txt
        """
    )

    parser.add_argument(
        "source_directory",
        help="Diretório de origem para ler a estrutura e arquivos"
    )

    parser.add_argument(
        "output_file", 
        help="Arquivo de destino onde salvar o conteúdo consolidado (.txt)"
    )

    # Parse dos argumentos
    if len(sys.argv) == 1:
        parser.print_help()
        return

    args = parser.parse_args()

    try:
        print(f"Lendo diretório: {args.source_directory}")
        content = read_directory_structure(args.source_directory)

        print(f"Escrevendo para: {args.output_file}")
        write_to_file(content, args.output_file)

        print("Processo concluído com sucesso!")

    except FileNotFoundError as e:
        print(f"Erro: {e}")
        sys.exit(1)
    except ValueError as e:
        print(f"Erro: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Erro inesperado: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
