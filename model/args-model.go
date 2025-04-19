/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package model

import (
	"slices"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/elliotchance/orderedmap/v3"
)

type Args struct {
	args       []string
	argm       *orderedmap.OrderedMap[string, any]
	Command    string
	IsOptHelp  bool
	IsOptWatch bool
	IsVersion  bool

	IsDocker bool
	IsPs     bool
	IsImages bool
	IsKube   bool
	IsInfo   bool
	IsAction bool
	IsApply  bool
	IsYml    bool
	IsDiff   bool
	IsDelete bool
	IsConf   bool
	IsSecret bool
	IsExec   bool
	IsPods   bool
	IsLs     bool
	IsWatch  bool
	IsLogs   bool
	IsEvents bool
}

func (slf *Args) SetArgs(args []string) {
	slf.args = args
	slf.argm = orderedmap.NewOrderedMap[string, any]()
	for _, arg := range args {
		slf.argm.Set(arg, nil)
	}

	if len(args) > 0 {
		slf.Command = strings.Join(args, " ")
	}
}

func (slf *Args) AddRemains(command string, excludeList ...string) string {
	remains := slf.GetRemains(command, excludeList...)
	if remains != "" {
		return command + " " + remains
	}
	return command
}

func (slf *Args) GetRemains(excludeCommand string, excludeList ...string) string {
	args := slices.Clone(slf.args)
	out := strings.Join(args, singleSpace)

	for strings.Contains(excludeCommand, doubleSpace) {
		excludeCommand = strings.ReplaceAll(excludeCommand, doubleSpace, singleSpace)
	}

	if excludeCommand != "" {
		switch {
		case out == excludeCommand:
			out = strings.TrimSpace(strings.Replace(out, excludeCommand, "", 1))

		case len(out) > len(excludeCommand) && out[:len(excludeCommand)] == excludeCommand:
			if out[len(excludeCommand):len(excludeCommand)+1] == singleSpace {
				out = strings.TrimSpace(strings.Replace(out, excludeCommand, "", 1))
			}
		}
	}

	if len(excludeList) > 0 {
		args := strings.Split(out, singleSpace)
		for _, removeArg := range excludeList {
			args = removeFirstArg(args, removeArg)
		}

		out = fm.Ternary(len(args) == 0, "", strings.Join(args, singleSpace))
	}

	return out
}

func (slf *Args) GetOptVal(command string, name string, altName ...string) (opt string, optVal string, val string) {
	ls := strings.Split(command, singleSpace)
	for _, arg := range ls {
		if opt != "" {
			val = arg
			break
		}

		if len(altName) > 0 && altName[0] != "" {
			if arg == name || arg == altName[0] {
				opt = arg
			}
		} else {
			if arg == name {
				opt = arg
			}
		}
	}

	optVal = opt
	if val != "" {
		optVal += " " + val
	}

	return
}

func removeFirstArg(args []string, removeArg string) []string {
	isRemoved := false
	ls := make([]string, 0)

	for _, arg := range args {
		if !isRemoved && arg == removeArg {
			isRemoved = true
			continue
		}

		ls = append(ls, arg)
	}

	return ls
}
