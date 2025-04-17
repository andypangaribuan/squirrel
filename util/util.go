/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package util

import (
	"errors"
	"fmt"
	"os"
	"os/signal"
	"reflect"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/apoorvam/goterminal"
)

func ExitWithCtrlC() {
	done := make(chan os.Signal, 1)
	signal.Notify(done, syscall.SIGINT, syscall.SIGTERM) // ctrl+C : exit
	go func() {
		<-done
		os.Exit(0)
	}()
}

func Watch(fn func() string) {
	writer := goterminal.New(os.Stdout)
	out := fn()

	for {
		_, err := fmt.Fprintln(writer, out)
		if err == nil {
			err = writer.Print()
		}

		if err != nil {
			time.Sleep(time.Second)
			writer.Reset()
			continue
		}

		time.Sleep(time.Second)
		out = fn()
		writer.Clear()
	}
}

func VTrim(ls []string, idx int) string {
	val := ls[idx]
	for strings.Contains(val, doubleSpace) {
		val = strings.ReplaceAll(val, doubleSpace, singleSpace)
	}

	return val
}

func Build(items [][]any, rightAlign ...int) string {
	length := make([]int, 0)
	lsItem := make([][]string, len(items))
	mapRightAlign := make(map[int]any, 0)

	for _, index := range rightAlign {
		mapRightAlign[index] = nil
	}

	if len(items) > 0 {
		length = make([]int, len(items[0]))
	}

	for i, ls := range items {
		lsItem[i] = make([]string, len(ls))
		for j, v := range ls {
			value := ""

			if v != nil {
				switch val := v.(type) {
				case string:
					value = val
				case *string:
					value = *val
				case int:
					value = strconv.Itoa(val)
				case *int:
					value = strconv.Itoa(*val)
				default:
					rev := reflect.ValueOf(val)
					if rev.Kind() == reflect.Ptr {
						value = fmt.Sprintf("%v", rev.Elem().Interface())
					} else {
						value = fmt.Sprintf("%v", val)
					}
				}
			}

			lsItem[i][j] = value
		}
	}

	for _, ls := range lsItem {
		for i, v := range ls {
			if length[i] < len(v) {
				length[i] = len(v)
			}
		}
	}

	out := ""
	for _, ls := range lsItem {
		line := ""
		for i, v := range ls {
			if i > 0 {
				line += doubleSpace
			}

			isRightAlign := false
			if _, ok := mapRightAlign[i]; ok {
				isRightAlign = true
			}

			v = addSpace(length[i], v, isRightAlign)
			line += v
		}

		if out != "" {
			out += "\n"
		}
		out += line
	}

	return out
}

func addSpace(maxSize int, value string, rightAlign bool) string {
	if rightAlign {
		for len(value) < maxSize {
			value = singleSpace + value
		}
	} else {
		for len(value) < maxSize {
			value += singleSpace
		}
	}

	return value
}

func PrintHelp(help string, isError ...bool) {
	fmt.Printf("%v\n\n", strings.TrimSpace(help))
	if len(isError) > 0 {
		if isError[0] {
			os.Exit(1)
		} else {
			os.Exit(0)
		}
	}
}

func IsFileExists(filePath string) bool {
	_, error := os.Stat(filePath)
	return !errors.Is(error, os.ErrNotExist)
}

func UnknownCommand(remainsCommand string, helpMessage string) {
	remainsCommand = strings.TrimSpace(remainsCommand)
	msg := helpMessage
	if remainsCommand != "" {
		msg = fmt.Sprintf("unknown command/options: %v\n\n%v", remainsCommand, msg)
	}

	PrintHelp(msg, true)
}
