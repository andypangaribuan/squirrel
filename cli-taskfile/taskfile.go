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
	"squirrel/util"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/wissance/stringFormatter"
)

func Exec() {
	filePath := fmt.Sprintf("%v/.taskfile", getWorkingDirectory())
	_, err := os.Stat(filePath)
	if errors.Is(err, os.ErrNotExist) {
		return
	}

	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("error: %v\n%+v\n\n", err, err)
		return
	}

	var (
		lines            = strings.Split(string(content), "\n")
		items            = make([][]any, 0)
		newLineAtIndexes = make([]int, 0)
	)

	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "#: space" {
			newLineAtIndexes = append(newLineAtIndexes, len(items)-1)
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
						items = append(items, []any{"", function, desc})
					}
				}
			}

		}
	}

	if len(items) > 0 {
		output := util.Build(items)
		if len(newLineAtIndexes) > 0 {
			lines := strings.Split(output, "\n")
			newLines := make([]string, 0)

			if fm.IfHaveIn(-1, newLineAtIndexes...) {
				newLines = append(newLines, "")
			}

			for i, line := range lines {
				newLines = append(newLines, line)
				if fm.IfHaveIn(i, newLineAtIndexes...) {
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
		fmt.Println(msg)
	}
}
