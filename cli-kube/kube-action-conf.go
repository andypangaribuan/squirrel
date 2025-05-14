/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package clikube

import (
	"fmt"
	"os"
	"squirrel/util"
	"strings"
	"sync"

	"github.com/andypangaribuan/gmod/fm"
)

func kubeActionConf(namespace string, appName string, lsYml []string) {
	var (
		wg         sync.WaitGroup
		workingDir = getWorkingDirectory()
		envs       = getEnvs(workingDir)

		output            string
		noResourceMessage = "no resources found"
		nilMessage        = util.ColorCyan("<NIL>")

		isHaveSA       = fm.IfHaveIn("sa", lsYml...)
		isHaveCM       = fm.IfHaveIn("cm", lsYml...)
		isHaveSecret   = fm.IfHaveIn("secret", lsYml...)
		isHaveDep      = fm.IfHaveIn("dep", lsYml...)
		isHavePDB      = fm.IfHaveIn("pdb", lsYml...)
		isHaveHPA      = fm.IfHaveIn("hpa", lsYml...)
		isHaveSVC      = fm.IfHaveIn("svc", lsYml...)
		isHaveING      = fm.IfHaveIn("ing", lsYml...)
		isHaveStateful = fm.IfHaveIn("stateful", lsYml...)
		isHavePV       = fm.IfHaveIn("pv", lsYml...)
		isHavePVC      = fm.IfHaveIn("pvc", lsYml...)

		outSA       string
		outCM       string
		outSecret   string
		outDep      string
		outPDB      string
		outHPA      string
		outSVC      string
		outING      string
		outStateful string
		outPV       string
		outPVC      string
	)

	exec := func(script string) string {
		out, err := util.Terminal("", script)
		if err != nil && !strings.Contains(strings.ToLower(*err), noResourceMessage) {
			fmt.Println(*err)
			os.Exit(1)
		}

		return out
	}

	updateOutput := func(isHave bool, title string, out string) {
		if isHave {
			if out == nilMessage {
				title = util.ColorBoldRed(title)
			} else {
				title = util.ColorBoldGreen(title)
			}

			output += fm.Ternary(output == "", "", "\n\n") +
				title + "\n" +
				out
		}
	}

	run := func(isHave bool, key string, ref *string) {
		if isHave {
			script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v -n %v", key, appName, namespace)
			if key == "pv" {
				script = fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v", key, appName)
			}

			wg.Add(1)

			go func() {
				out := exec(script)

				if key == "svc" {
					if out != "" {
						endpoint := ""

						keys, vals := util.MapKV(out, "NAME", "TYPE", "CLUSTER-IP", "EXTERNAL-IP", "PORT(S)", "AGE")
						if len(vals) > 0 {
							if index, ok := keys["CLUSTER-IP"]; ok {
								clusterIp := strings.ToLower(vals[0][index])
								if clusterIp == "none" {
									script := fmt.Sprintf("kubectl get ep --field-selector metadata.name=%v -n %v", appName, namespace)
									out := exec(script)
									if out != "" {
										keys, vals := util.MapKV(out, "NAME", "ENDPOINTS", "AGE")
										if index, ok := keys["ENDPOINTS"]; ok {
											for _, v := range vals {
												if endpoint != "" {
													endpoint += ", "
												}
												endpoint += v[index]
											}
										}
									}
								}
							}
						}

						out = strings.TrimSpace(out)
						if endpoint != "" {
							out += "\n" + util.ColorCyan("ep ") + util.ColorYellow(endpoint)
						}

						if namespace != "" && appName != "" {
							port := util.ColorCyan("{port}")
							arrow := util.ColorCyan(fm.Ternary(endpoint == "", "", " ") + "→ ")
							out += "\n" + arrow + util.ColorYellow(appName+"."+namespace+":") + port +
								"\n" + arrow + util.ColorYellow(appName+"."+namespace+".svc.cluster.local:") + port
						}
					}
				}

				if key == "ing" {
					if out == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-grpc -n %v", key, appName, namespace)
						out = exec(script)
					}
				}

				if key == "pv" {
					if out == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-pv", key, appName)
						out = exec(script)
					}

					if out == "" {
						if name, ok := envs[keyKymlPvName]; ok {
							script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v", key, name)
							out = exec(script)
						}
					}

					if out == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-%v-pv", key, appName, namespace)
						out = exec(script)
					}
				}

				if key == "pvc" {
					if out == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-pvc -n %v", key, appName, namespace)
						out = exec(script)
					}

					if out == "" {
						if name, ok := envs[keyKymlPvcName]; ok {
							script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v -n %v", key, name, namespace)
							out = exec(script)
						}
					}

					if out == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v%v-pvc -n %v", key, appName, namespace, namespace)
						out = exec(script)
					}
				}

				out = strings.TrimSpace(out)
				*ref = fm.Ternary(out == "", nilMessage, out)

				wg.Done()
			}()
		}
	}

	run(isHaveSA, "sa", &outSA)
	run(isHaveCM, "cm", &outCM)
	run(isHaveSecret, "secret", &outSecret)
	run(isHaveDep, "deploy", &outDep)
	run(isHavePDB, "pdb", &outPDB)
	run(isHaveHPA, "hpa", &outHPA)
	run(isHaveSVC, "svc", &outSVC)
	run(isHaveING, "ing", &outING)
	run(isHaveStateful, "statefulset", &outStateful)
	run(isHavePV, "pv", &outPV)
	run(isHavePVC, "pvc", &outPVC)

	wg.Wait()

	for _, yml := range lsYml {
		switch yml {
		case "sa":
			updateOutput(isHaveSA, "SERVICE ACCOUNT", outSA)
		case "cm":
			updateOutput(isHaveCM, "CONFIG MAP", outCM)
		case "secret":
			updateOutput(isHaveSecret, "SECRET", outSecret)
		case "dep":
			updateOutput(isHaveDep, "DEPLOYMENT", outDep)
		case "pdb":
			updateOutput(isHavePDB, "POD DISRUPTION BUDGET", outPDB)
		case "hpa":
			updateOutput(isHaveHPA, "HORIZONTAL POD AUTOSCALER", outHPA)
		case "svc":
			updateOutput(isHaveSVC, "SERVICES", outSVC)
		case "ing":
			updateOutput(isHaveING, "INGRESS", outING)
		case "stateful":
			updateOutput(isHaveStateful, "STATEFUL SET", outStateful)
		case "pv":
			updateOutput(isHavePV, "PERSISTENT VOLUME", outPV)
		case "pvc":
			updateOutput(isHavePVC, "PERSISTENT VOLUME CLAIM", outPVC)
		}
	}

	fmt.Printf("%v\n\n", output)
	os.Exit(0)
}
