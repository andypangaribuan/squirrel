/*
 * Copyright (c) 2024.
 * Created by Andy Pangaribuan <https://github.com/apangaribuan>.
 * All Rights Reserved.
 */

package util

import (
	"bytes"
	"fmt"
	"os/exec"
	"strconv"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
)

func ToPath(args []string, availableCommands string, path map[string]func(args []string)) {
	ls := strings.Split(strings.TrimSpace(availableCommands), "\n")
	availableCommands = ""

	for i, v := range ls {
		if i != 0 {
			availableCommands += "\n"
		}

		availableCommands += strings.TrimSpace(v)
	}

	if len(args) == 0 {
		fmt.Printf("invalid command\n\navailable commands:\n%v\n", availableCommands)
		return
	}

	fn, ok := path[args[0]]
	if ok {
		fn(args[1:])
	} else {
		fmt.Printf("invalid command\n%v\n", availableCommands)
	}
}

func PrintInvalidCommand() {
	fmt.Printf("invalid command\n")
}

func GetFirstOrEmpty(args []string) string {
	if len(args) == 0 {
		return ""
	}

	return args[0]
}

func CMD(sh string, loadEnv ...bool) (string, string) {
	if len(loadEnv) > 0 && loadEnv[0] {
		sh = "set -a; source ~/.zshrc; set +a; " + sh
	}

	var stdout, stderr bytes.Buffer
	cmd := exec.Command("bash", "-c", sh)
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	_ = cmd.Run()

	return stdout.String(), stderr.String()
}

func GetVO(args []string) ([]string, []string, [][]string, []string, [][]string) {
	var (
		val  = make([]string, 0)
		opt  = make([]string, 0)
		opts = make([][]string, 0)
		ext  = make([]string, 0)
		exts = make([][]string, 0)
	)

	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch {
		case arg[:1] == "-" && strings.Contains(arg, "="):
			ls := strings.Split(arg, "=")
			opts = append(opts, []string{ls[0], ls[1]})

		case arg[:1] == "-":
			opt = append(opt, arg)

		case arg[:1] == "+" && strings.Contains(arg, "="):
			ls := strings.Split(arg, "=")
			exts = append(exts, []string{ls[0], ls[1]})

		case arg[:1] == "+":
			ext = append(ext, arg)

		default:
			val = append(val, arg)
		}
	}

	return val, opt, opts, ext, exts
}

func AddSHO(sh string, opt []string, opts [][]string) string {
	for _, v := range opt {
		sh += fm.Ternary(sh == "", "", " ")
		sh += v
	}

	for _, v := range opts {
		sh += fm.Ternary(sh == "", "", " ")
		sh += fmt.Sprintf("%v %v", v[0], v[1])
	}

	return sh
}

func FullSHO(opt []string, opts [][]string, sh string, args ...any) string {
	return AddSHO(fmt.Sprintf(sh, args...), opt, opts)
}

func StrClean(val string) string {
	for {
		val = strings.TrimSpace(val)
		if len(val) == 0 {
			break
		}

		if val[:1] == "\n" {
			val = val[1:]
			continue
		}

		if val[len(val)-1:] == "\n" {
			val = val[:len(val)-1]
			continue
		}

		break
	}

	return val
}

func GetOptsVal(opts [][]string, key string) string {
	for _, opt := range opts {
		if opt[0] == key {
			return opt[1]
		}
	}

	return ""
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

func AddNumberHV(header map[string]int, vals [][]string) (map[string]int, [][]string) {
	newHeader := make(map[string]int, 0)
	newVals := make([][]string, 0)

	newHeader["NO"] = 0
	for k, v := range header {
		newHeader[k] = v + 1
	}

	for i, v := range vals {
		vs := []string{strconv.Itoa(i + 1)}
		vs = append(vs, v...)
		newVals = append(newVals, vs)
	}

	return newHeader, newVals
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
			// for line[nx:nx+1] != " " {
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

func GetMax(values [][]string) []int {
	max := make([]int, len(values))
	for _, ls := range values {
		for i, v := range ls {
			cur := max[i]
			max[i] = fm.Ternary(cur < len(v), len(v), cur)
		}
	}
	return max
}

func GetMaxVals(vals ...int) int {
	max := 0
	for _, v := range vals {
		max = fm.Ternary(max < v, v, max)
	}
	return max
}

func BuildLines(lines [][]string) string {
	lengths := make(map[int]int, 0)
	for i, line := range lines {
		if i == 0 {
			for i, item := range line {
				lengths[i] = len(item)
			}
		} else {
			for i, item := range line {
				lengths[i] = fm.Ternary(lengths[i] > len(item), lengths[i], len(item))
			}
		}
	}

	vs := make([]string, 0)
	for _, line := range lines {
		v := ""
		for i, item := range line {
			if i == len(line)-1 {
				v += item
			} else {
				v += AddSpace(item, lengths[i]) + " "
			}
		}

		vs = append(vs, v)
	}

	return strings.Join(vs[:], "\n")
}

func AddSpace(val string, length int, onLeft ...bool) string {
	left := *fm.GetFirst(onLeft, false)
	for len(val) >= length {
		val = fm.Ternary(left, " "+val, val+" ")
	}
	return val
}

func ExtExists(vals []string, key string) bool {
	for _, val := range vals {
		if val == key {
			return true
		}
	}

	return false
}

func FindExtVal(vals [][]string, key string) (string, bool) {
	for _, val := range vals {
		if val[0] == key {
			return val[1], true
		}
	}

	return "", false
}
