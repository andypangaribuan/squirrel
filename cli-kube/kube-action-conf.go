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
		outIngGrpc  string
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

	updateOutput := func(isHave bool, title string, out1 string, out2 string) {
		if isHave {
			out := nilMessage

			if out1 == nilMessage && out2 == nilMessage {
				title = util.ColorBoldRed(title)
			} else {
				title = util.ColorBoldGreen(title)
			}

			switch {
			case out1 != nilMessage && out2 != "":
				out = out1 + "\n" + out2
			case out1 != nilMessage:
				out = out1
			case out2 != "":
				out = out2
			}

			output += fm.Ternary(output == "", "", "\n\n") +
				title + "\n" +
				out
		}
	}

	run := func(isHave bool, key string, ref1 *string, ref2 *string) {
		if isHave {
			script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v -n %v", key, appName, namespace)
			if key == "pv" {
				script = fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v", key, appName)
			}

			wg.Add(1)

			go func() {
				out1 := exec(script)
				out2 := ""

				if key == "svc" {
					if out1 != "" {
						endpoint := ""

						keys, vals := util.MapKV(out1, "NAME", "TYPE", "CLUSTER-IP", "EXTERNAL-IP", "PORT(S)", "AGE")
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

						out1 = strings.TrimSpace(out1)
						if endpoint != "" {
							out1 += "\n" + util.ColorCyan("ep ") + util.ColorYellow(endpoint)
						}

						if namespace != "" && appName != "" {
							port := util.ColorCyan("{port}")
							arrow := util.ColorCyan(fm.Ternary(endpoint == "", "", " ") + "→ ")
							out1 += "\n" + arrow + util.ColorYellow(appName+"."+namespace+":") + port +
								"\n" + arrow + util.ColorYellow(appName+"."+namespace+".svc.cluster.local:") + port
						}
					}
				}

				if key == "ing" {
					script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-grpc -n %v", key, appName, namespace)
					out2 = exec(script)
				}

				if key == "pv" {
					if out1 == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-pv", key, appName)
						out1 = exec(script)
					}

					if out1 == "" {
						if name, ok := envs[keyKymlPvName]; ok {
							script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v", key, name)
							out1 = exec(script)
						}
					}

					if out1 == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-%v-pv", key, appName, namespace)
						out1 = exec(script)
					}
				}

				if key == "pvc" {
					if out1 == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v-pvc -n %v", key, appName, namespace)
						out1 = exec(script)
					}

					if out1 == "" {
						if name, ok := envs[keyKymlPvcName]; ok {
							script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v -n %v", key, name, namespace)
							out1 = exec(script)
						}
					}

					if out1 == "" {
						script := fmt.Sprintf("kubectl get %v --field-selector metadata.name=%v%v-pvc -n %v", key, appName, namespace, namespace)
						out1 = exec(script)
					}
				}

				out1 = strings.TrimSpace(out1)
				out2 = strings.TrimSpace(out2)

				*ref1 = fm.Ternary(out1 == "", nilMessage, out1)
				if ref2 != nil && out2 != "" {
					*ref2 = out2
				}

				wg.Done()
			}()
		}
	}

	run(isHaveSA, "sa", &outSA, nil)
	run(isHaveCM, "cm", &outCM, nil)
	run(isHaveSecret, "secret", &outSecret, nil)
	run(isHaveDep, "deploy", &outDep, nil)
	run(isHavePDB, "pdb", &outPDB, nil)
	run(isHaveHPA, "hpa", &outHPA, nil)
	run(isHaveSVC, "svc", &outSVC, nil)
	run(isHaveING, "ing", &outING, &outIngGrpc)
	run(isHaveStateful, "statefulset", &outStateful, nil)
	run(isHavePV, "pv", &outPV, nil)
	run(isHavePVC, "pvc", &outPVC, nil)

	wg.Wait()

	for _, yml := range lsYml {
		switch yml {
		case "sa":
			updateOutput(isHaveSA, "SERVICE ACCOUNT", outSA, "")
		case "cm":
			updateOutput(isHaveCM, "CONFIG MAP", outCM, "")
		case "secret":
			updateOutput(isHaveSecret, "SECRET", outSecret, "")
		case "dep":
			updateOutput(isHaveDep, "DEPLOYMENT", outDep, "")
		case "pdb":
			updateOutput(isHavePDB, "POD DISRUPTION BUDGET", outPDB, "")
		case "hpa":
			updateOutput(isHaveHPA, "HORIZONTAL POD AUTOSCALER", outHPA, "")
		case "svc":
			updateOutput(isHaveSVC, "SERVICES", outSVC, "")
		case "ing":
			updateOutput(isHaveING, "INGRESS", outING, outIngGrpc)
		case "stateful":
			updateOutput(isHaveStateful, "STATEFUL SET", outStateful, "")
		case "pv":
			updateOutput(isHavePV, "PERSISTENT VOLUME", outPV, "")
		case "pvc":
			updateOutput(isHavePVC, "PERSISTENT VOLUME CLAIM", outPVC, "")
		}
	}

	fmt.Printf("%v\n\n", output)
	os.Exit(0)
}
