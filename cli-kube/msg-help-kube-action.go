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
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

// sq kube action --help
var msgHelpKubeAction = stringFormatter.FormatComplex(`
info : Comprehensive kubectl execution
usage: sq kube action [commands]

{commands}
  apply         Apply yml configuration to kubernetes
  yml           Show yml file
  diff          Show different between local and kubernetes
  delete        Delete yml configuration on kubernetes

  conf          Show all configurations
  secret        Show all decoded secret
  exec {name}   Go to shell pod (default: first pod)
  pods          Execute pods cli

{options}
  --namespace {name1}      Application namespace, when value start with {kyml} then get from .env file
  --app {name1}            Application name, when value start with {kyml} then get from .env file
  --yml {vals}            Execution of yaml file (csv format, without space)
                          Values: {ymls}
  --yml-template {vals}   Last yml file used when --yml not found (csv format, without space)
                          Search from os environment {template-dir} (directory path)
                          Use {separator} to separate your key, and values equals to --yml value
                          e.q. {yml-template} {sa1},{svc1}
                          [os env] {export}
                          Yml file inside template directory: {sa2}, {svc2}
`, map[string]any{
	"commands":     util.ColorBoldGreen("commands:"),
	"options":      util.ColorBoldGreen("options:"),
	"name1":        util.ColorCyan("{name}"),
	"ymls":         util.ColorYellow("sa, cm, secret, dep, pdb, hpa, svc, ing, stateful, pv, pvc"),
	"vals":         util.ColorCyan("{vals}"),
	"kyml":         util.ColorYellow("KYML_"),
	"template-dir": util.ColorYellow("'SQ_CLI_TEMPLATE_DIR'"),
	"export":       util.ColorCyan("export SQ_CLI_TEMPLATE_DIR=/path/to/your/template/directory"),
	"separator":    util.ColorBoldRed("'-'"),
	"yml-template": util.ColorYellow("--yml-template"),
	"sa1":          util.ColorBoldRed("sa"),
	"sa2":          util.ColorBoldRed("sa.yml"),
	"svc1":         util.ColorBoldRed("svc-rest"),
	"svc2":         util.ColorBoldRed("svc-rest.yml"),
})
