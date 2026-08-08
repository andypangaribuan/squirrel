/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package kube

import (
	"fmt"
	"os"
	"regexp"
	"squirrel/app"
	"squirrel/util"
	"strings"

	"github.com/andypangaribuan/gmod/gm"
	"github.com/joho/godotenv"
)

func getEnvs(workingDir string) (envs map[string]string) {
	var err error
	envs = make(map[string]string, 0)

	envFile := fmt.Sprintf("%v/.env", workingDir)
	if util.IsFileExists(envFile) {
		envs, err = godotenv.Read(envFile)
		if err != nil {
			fmt.Printf("%+v\n", err)
			os.Exit(1)
		}
	}

	return envs
}

func getKymlOsEnvs() (envs map[string]string) {
	envs = make(map[string]string, 0)

	for _, e := range os.Environ() {
		pair := strings.SplitN(e, "=", 2)
		if len(pair) > 0 {
			key := pair[0]
			if len(key) > 5 && key[:5] == "KYML_" {
				val, ok := os.LookupEnv(key)
				if ok {
					envs[key] = val
				}
			}
		}
	}

	return
}

func getWorkingDirectory() string {
	workingDir, err := os.Getwd()
	if err != nil {
		fmt.Printf("%+v\n", err)
		os.Exit(1)
	}

	return workingDir
}

func getSqCliOsEnvs() (envs map[string]string) {
	envs = make(map[string]string, 0)

	for _, e := range os.Environ() {
		pair := strings.SplitN(e, "=", 2)
		if len(pair) > 0 {
			key := pair[0]
			if len(key) > 7 && key[:7] == "SQ_CLI_" {
				val, ok := os.LookupEnv(key)
				if ok {
					envs[key] = val
				}
			}
		}
	}

	return
}

func getYmlFilePath(ymlTemplates []string, workingDir string, optVal string, level int) (string, string) {
	if level > searchFileMaxLevelAbove {
		ymlTemplate := ""

		for _, template := range ymlTemplates {
			if template == optVal {
				ymlTemplate = template
				break
			}

			if len(template) > len(optVal) && template[:len(optVal)] == optVal {
				ymlTemplate = template
				break
			}
		}

		if ymlTemplate != "" {
			templateDir, ok := getSqCliOsEnvs()["SQ_CLI_TEMPLATE_DIR"]
			if ok {
				filePath := templateDir + string(os.PathSeparator) + ymlTemplate + ".yml"
				if util.IsFileExists(filePath) {
					return filePath, ""
				}

				filePath = templateDir + string(os.PathSeparator) + ymlTemplate + ".yaml"
				if util.IsFileExists(filePath) {
					return filePath, ""
				}
			}

			command := "curl -s " + app.GithubTemplateDirectory + ymlTemplate + ".yml"
			out, err := util.Terminal("", command)
			if err == nil && out != "" && len(out) > 3 && out[:3] != "404" {
				return "", out
			}
		}

		return "", ""
	}

	filePath := fmt.Sprintf("%v/%v.yml", workingDir, optVal)
	if util.IsFileExists(filePath) {
		return filePath, ""
	}

	if optVal == "dep" {
		filePath = fmt.Sprintf("%v/deploy.yml", workingDir)
		if util.IsFileExists(filePath) {
			return filePath, ""
		}

		filePath = fmt.Sprintf("%v/deployment.yml", workingDir)
		if util.IsFileExists(filePath) {
			return filePath, ""
		}
	}

	separator := string(os.PathSeparator)
	ls := strings.Split(workingDir, separator)
	ls = ls[:len(ls)-1]
	workingDir = strings.Join(ls, separator)

	return getYmlFilePath(ymlTemplates, workingDir, optVal, level+1)
}

func replaceWithEnv(lines string) string {
	envs := getEnvs(getWorkingDirectory())
	tempReplace := gm.Util.UID(100)

	for {
		replaced := 0
		for key, val := range envs {
			re := regexp.MustCompile("\\b" + key + "\\b")
			if re != nil {
				for re.MatchString(lines) {
					replaced++
					lines = re.ReplaceAllString(lines, tempReplace)
					lines = strings.ReplaceAll(lines, "$"+tempReplace, val)
				}
			}
		}

		if replaced == 0 {
			break
		}
	}

	return lines
}

func replaceWithKymlOsEnvs(lines string) string {
	kymlOsEnvs := getKymlOsEnvs()

	for {
		replaced := 0

		for key, val := range kymlOsEnvs {
			key = "$" + key
			for strings.Contains(lines, key) {
				replaced++
				lines = strings.ReplaceAll(lines, key, val)
			}
		}

		if replaced == 0 {
			break
		}
	}

	return lines
}

func getYmlLines(ymlName string, ymlTemplates []string) string {
	workingDir := getWorkingDirectory()
	ymlFile, lines := getYmlFilePath(ymlTemplates, workingDir, ymlName, 1)
	if ymlFile == "" && lines == "" {
		fmt.Printf("cannot find %v.yml file (up to %v level above)\n", ymlName, searchFileMaxLevelAbove)
		os.Exit(1)
	}

	if ymlFile != "" {
		data, err := os.ReadFile(ymlFile)
		if err != nil {
			fmt.Printf("%+v\n", err)
			os.Exit(1)
		}

		lines = string(data)
	}

	lines = replaceWithEnv(lines)
	lines = replaceWithKymlOsEnvs(lines)

	if ymlName == "cm" || ymlName == "secret" {
		lines = replaceWithEnv(lines)
	}

	return lines
}
