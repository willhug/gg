package cli

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

func NewRunner(cmd string) *Runner {
	return &Runner{
		command: cmd,
	}
}

type Runner struct {
	command string
}

// Run but don't fail
func (r *Runner) SoftRun(subcommand string, params ...string) {
	err := r.Run(subcommand, params...)
	fmt.Println("swallowing error from run: ", err.Error())
}

func (r *Runner) Run(subcommand string, params ...string) error {
	return r.runToStdOut(subcommand, params...)
}

func (r *Runner) RunWithSingleResp(subcommand string, params ...string) (string, error) {
	buf, err := r.runWithOutput(subcommand, params...)
	if err != nil {
		return "", err
	}
	return strings.Trim(buf.String(), "\t\n\r"), nil
}

func (r *Runner) RunAndGetLines(subcommand string, params ...string) ([]string, error) {
	buf, err := r.runWithOutput(subcommand, params...)
	if err != nil {
		return nil, err
	}
	return strings.Split(buf.String(), "\n"), nil
}

func (r *Runner) RunAndGetAllAsString(subcommand string, params ...string) (string, error) {
	buf, err := r.runWithOutput(subcommand, params...)
	if err != nil {
		return "", err
	}
	return buf.String(), nil
}

func (r *Runner) runToStdOut(subcommand string, params ...string) error {
	args := append([]string{subcommand}, params...)
	cmd := exec.Command(r.command, args...)

	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	return cmd.Run()
}

func (r *Runner) runWithOutput(subcommand string, params ...string) (*bytes.Buffer, error) {
	args := append([]string{subcommand}, params...)
	cmd := exec.Command(r.command, args...)

	buf := bytes.NewBuffer(nil)
	cmd.Stdout = buf
	cmd.Stderr = os.Stderr

	return buf, cmd.Run()
}
