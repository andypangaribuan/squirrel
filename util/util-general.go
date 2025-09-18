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
	"regexp"
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

func Split(value string, separator string) []string {
	value = strings.TrimSpace(value)
	if value == "" {
		return []string{}
	}

	return strings.Split(value, separator)
}

func VTrim(ls []string, idx int) string {
	val := ls[idx]
	for strings.Contains(val, doubleSpace) {
		val = strings.ReplaceAll(val, doubleSpace, singleSpace)
	}

	return val
}

func MapKV(val string, headerNames ...string) (map[string]int, [][]string) {
	val = strings.TrimSpace(val)

	var (
		indexes = make(map[string][]int, 0)
		header  = make(map[string]int, 0)
		vals    = make([][]string, 0)
	)

	lines := strings.Split(val, "\n")
	if val != "" {
		for i, line := range lines {
			if i == 0 {
				indexes = getHeaderIndex(line, headerNames)
				for i, h := range headerNames {
					header[h] = i
				}
			} else {
				values := make([]string, 0)
				for _, h := range headerNames {
					idx := indexes[h]
					v := ""
					if len(idx) == 1 {
						v = line[idx[0]:]
					} else {
						v = line[idx[0]:idx[1]]
					}
					values = append(values, strings.TrimSpace(v))
				}
				vals = append(vals, values)
			}
		}
	}

	return header, vals
}

func getHeaderIndex(line string, headers []string) map[string][]int {
	var (
		length    = len(headers)
		indexes   = make(map[string][]int, 0)
		idx       int
		nx        int
		prevCount = 0
	)

	for i, header := range headers {
		idx = strings.Index(line, header)
		if i == length-1 {
			indexes[header] = []int{prevCount + idx}
		} else {
			nx = idx + len(header)
			for line[nx:nx+1] == " " {
				nx++
			}
			indexes[header] = []int{prevCount + idx, prevCount + nx}
			prevCount += nx
			line = line[nx:]
		}
	}

	return indexes
}

func CalcI0MaxLine(items [][]string) int {
	i0MaxLine := 0

	for _, ls := range items {
		if len(ls[0]) > i0MaxLine {
			i0MaxLine = len(ls[0])
		}
	}

	return i0MaxLine
}

func TwoCenter(items [][]string, leftSpace string, minSeparator string, forceI0MaxLine int) string {
	i0MaxLine := 0

	for _, ls := range items {
		if len(ls[0]) > i0MaxLine {
			i0MaxLine = len(ls[0])
		}
	}

	if forceI0MaxLine > 0 && forceI0MaxLine > i0MaxLine {
		i0MaxLine = forceI0MaxLine
	}

	lsItem := make([]string, len(items))
	for i, ls := range items {
		line1 := addSpace(i0MaxLine, ls[0], false)
		line1 += minSeparator
		line2 := ls[1]
		lsItem[i] = leftSpace + line1 + line2
	}

	return strings.Join(lsItem, "\n")
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

func IsFileExists(filePath string) bool {
	_, error := os.Stat(filePath)
	return !errors.Is(error, os.ErrNotExist)
}

func ContainsOnlyAlphanumericAndUnderscore(s string) bool {
	pattern := "^[a-zA-Z0-9_]*$"
	re := regexp.MustCompile(pattern)
	return re.MatchString(s)
}


func ContainsOnlyNumeric(s string) bool {
	pattern := "^[0-9]*$"
	re := regexp.MustCompile(pattern)
	return re.MatchString(s)
}
