/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package clitaskfile

import (
	"errors"
	"fmt"
	"os"
	"squirrel/app"
	"squirrel/util"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/wissance/stringFormatter"
)

func Exec() {
	var (
		filePaths = []string{fmt.Sprintf("%v/.taskfile", getWorkingDirectory())}
		args      = app.Args.Args()
		model     = &stuTaskfile{
			items:            make([][]any, 0),
			newLineAtIndexes: make([]int, 0),
		}
	)

	if len(args) > 1 {
		for i, arg := range args {
			if arg == "taskfile" {
				continue
			}

			if i == 1 && arg != ".taskfile" {
				filePaths[0] = arg
			} else {
				filePaths = append(filePaths, arg)
			}
		}
	}

	for _, filePath := range filePaths {
		err := fileOutput(filePath, model)
		if err != nil {
			return
		}
	}

	printOutput(model)
}

func fileOutput(filePath string, model *stuTaskfile) error {
	_, err := os.Stat(filePath)
	if errors.Is(err, os.ErrNotExist) {
		return err
	}

	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("error: %v\n%+v\n\n", err, err)
		return err
	}

	lines := strings.Split(string(content), "\n")

	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "#: space" {
			model.newLineAtIndexes = append(model.newLineAtIndexes, len(model.items)-1)
			continue
		}

		if len(line) > 3 && line[:3] == "#: " && len(line[3:]) > 0 {
			if len(lines) > i+2 {
				nextLine := strings.TrimSpace(lines[i+1])
				if len(nextLine) > 10 && nextLine[:8] == "function" && nextLine[len(nextLine)-1:] == "{" {
					ls := strings.Split(nextLine, " ")
					if len(ls) == 3 {
						desc := strings.TrimSpace(line[3:])
						function := ls[1] + " "
						model.items = append(model.items, []any{"", function, desc})
					}
				}
			}

		}
	}

	return nil
}

func printOutput(model *stuTaskfile) {
	if len(model.items) == 0 {
		return
	}

	output := util.Build(model.items)
	if len(model.newLineAtIndexes) > 0 {
		lines := strings.Split(output, "\n")
		newLines := make([]string, 0)

		if fm.IfHaveIn(-1, model.newLineAtIndexes...) {
			newLines = append(newLines, "")
		}

		for i, line := range lines {
			newLines = append(newLines, line)
			if fm.IfHaveIn(i, model.newLineAtIndexes...) {
				newLines = append(newLines, "")
			}
		}

		output = strings.Join(newLines, "\n")
	}

	msg := stringFormatter.FormatComplex(`
{commands}
{output}`,
		map[string]any{
			"commands": util.ColorBoldGreen("commands:"),
			"output":   output,
		})

	if len(msg) > 1 && msg[:1] == "\n" {
		msg = msg[1:]
	}

	fmt.Println(msg)
}
