using System.Diagnostics;
using System.Security.Cryptography;
using Nemu.LowIl;

static void Require(bool condition, string message)
{
    if (!condition) throw new InvalidOperationException(message);
}
static void Denied(Action operation, string description)
{
    try { operation(); }
    catch (UnauthorizedAccessException) { return; }
    throw new InvalidOperationException($"Low write unexpectedly allowed: {description}");
}
static Process Start(string executable, params string[] arguments)
{
    var start = new ProcessStartInfo(executable) { UseShellExecute = false, CreateNoWindow = true };
    foreach (var value in arguments) start.ArgumentList.Add(value);
    return Process.Start(start) ?? throw new InvalidOperationException("Process not created.");
}

if (args.Length > 0 && args[0] == "--linger")
{
    File.WriteAllText(Path.Combine(args[1], "descendant-ready"), Environment.ProcessId.ToString());
    Thread.Sleep(30000);
    File.WriteAllText(Path.Combine(args[1], "escaped-job"), "FAIL");
    return 0;
}
if (args.Length > 0 && args[0] == "--probe")
{
    var temporary = Environment.GetEnvironmentVariable("TEMP")!;
    try
    {
        using var self = Process.GetCurrentProcess();
        Require(NativeSecurity.Integrity(self.Handle) == "S-1-16-4096", "worker IL");
        Require(NativeSecurity.PathIntegrity(temporary) == "S-1-16-4096", "temporary label");
        Require(Environment.GetEnvironmentVariable("P505_SECRET_SENTINEL") == null, "environment leak");
        Require(Environment.CurrentDirectory == temporary, "working directory");
        var expected = new[] { "", "two words", "trailing\\", "a\\\"b", "中文", "$(never-run)", "a;b" };
        Require(args.Skip(2).SequenceEqual(expected), "argument roundtrip");
        var fixture = args[1];
        foreach (var name in new[] { "core.sqlite", "core.sqlite-wal", "core.sqlite-shm" })
        {
            var file = Path.Combine(fixture, name);
            Denied(() => File.AppendAllText(file, "bad"), $"append {name}");
            Denied(() => File.WriteAllText(file, "bad"), $"overwrite {name}");
            Denied(() => File.Delete(file), $"delete {name}");
            Denied(() => File.Move(file, file + ".moved"), $"rename {name}");
        }
        Denied(() => File.WriteAllText(Path.Combine(fixture, "new.db"), "bad"), "create");
        Denied(() => Directory.CreateDirectory(Path.Combine(fixture, "bad")), "mkdir");
        var writable = Path.Combine(temporary, "positive-control");
        File.WriteAllText(writable, "low write works");
        Require(File.ReadAllText(writable) == "low write works", "low positive control");
        Require(NativeSecurity.PathIntegrity(writable) == "S-1-16-4096", "file label inheritance");
        var nested = Directory.CreateDirectory(Path.Combine(temporary, "nested")).FullName;
        File.WriteAllText(Path.Combine(nested, "positive"), "low nested write");
        Require(NativeSecurity.PathIntegrity(nested) == "S-1-16-4096", "directory label inheritance");
        using var descendant = Start(Environment.ProcessPath!, "--linger", temporary);
        var watch = Stopwatch.StartNew();
        while (!File.Exists(Path.Combine(temporary, "descendant-ready")) && watch.ElapsedMilliseconds < 10000)
            Thread.Sleep(25);
        Require(File.Exists(Path.Combine(temporary, "descendant-ready")), "descendant startup");
        File.WriteAllText(Path.Combine(temporary, "probe-result"), "PASS");
        return 0;
    }
    catch (Exception error)
    {
        File.WriteAllText(Path.Combine(temporary, "probe-result"), error.ToString());
        return 1;
    }
}

Require(args.Length == 2, "Expected absolute launcher EXE and new test root.");
NativeSecurity.RequireMedium();
var launcher = Path.GetFullPath(args[0]);
var root = Path.GetFullPath(args[1]);
Require(!Directory.Exists(root), "Test root must be new.");
Directory.CreateDirectory(root);
var fixtureDir = Directory.CreateDirectory(Path.Combine(root, "medium-fixture")).FullName;
Require(NativeSecurity.PathIntegrity(fixtureDir) == "S-1-16-8192", "fixture Medium label");
var files = new[] { "core.sqlite", "core.sqlite-wal", "core.sqlite-shm" };
var hashes = new Dictionary<string, string>();
foreach (var file in files)
{
    var path = Path.Combine(fixtureDir, file);
    File.WriteAllText(path, "Synthetic permission fixture, not an actual database.");
    hashes[file] = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)));
}
Console.WriteLine("PASS: medium positive writes; fixture effective IL=Medium (synthetic files).");

var start = new ProcessStartInfo(launcher) {
    UseShellExecute = false, CreateNoWindow = true, RedirectStandardOutput = true, RedirectStandardError = true };
var inputs = new[] { "--temp-parent", root, "--", Environment.ProcessPath!, "--probe", fixtureDir,
    "", "two words", "trailing\\", "a\\\"b", "中文", "$(never-run)", "a;b" };
foreach (var value in inputs) start.ArgumentList.Add(value);
start.Environment["P505_SECRET_SENTINEL"] = "test-only-not-a-secret";
using var run = Process.Start(start) ?? throw new InvalidOperationException("Launcher failed to start.");
var output = run.StandardOutput.ReadToEndAsync();
var errors = run.StandardError.ReadToEndAsync();
if (!run.WaitForExit(30000)) { run.Kill(entireProcessTree: true); throw new TimeoutException("Launcher timeout."); }
var stdout = await output;
var stderr = await errors;
File.WriteAllText(Path.Combine(root, "launcher.stdout.txt"), stdout);
File.WriteAllText(Path.Combine(root, "launcher.stderr.txt"), stderr);
Require(run.ExitCode == 0, $"Launcher returned {run.ExitCode}; inspect {root}.");
var tempLine = stdout.Split('\n').Single(line => line.StartsWith("worker_temp=", StringComparison.Ordinal));
var lowDirectory = tempLine.Trim()["worker_temp=".Length..];
Require(File.ReadAllText(Path.Combine(lowDirectory, "probe-result")) == "PASS", "worker probe failed");
foreach (var file in files)
    Require(hashes[file] == Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(Path.Combine(fixtureDir, file)))),
        "Medium fixture changed");
Require(Directory.GetFileSystemEntries(fixtureDir).Length == 3, "Medium fixture entries changed");
Require(NativeSecurity.PathIntegrity(fixtureDir) == "S-1-16-8192", "Medium label changed");
Console.WriteLine("PASS: Low token; Low temp write/inheritance; 14 Medium write attempts denied; fixture hashes unchanged.");
Console.WriteLine("PASS: argument roundtrip; inherited secret environment removed.");
var descendantId = int.Parse(File.ReadAllText(Path.Combine(lowDirectory, "descendant-ready")));
try
{
    using var descendant = Process.GetProcessById(descendantId);
    Require(descendant.WaitForExit(5000), "Descendant survived launcher exit.");
}
catch (ArgumentException) { /* Already exited. */ }
Require(!File.Exists(Path.Combine(lowDirectory, "escaped-job")), "Descendant escaped job");
Console.WriteLine("PASS: surviving descendant terminated on launcher job close.");
using var parent = Process.GetCurrentProcess();
Require(NativeSecurity.Integrity(parent.Handle) == "S-1-16-8192", "Parent integrity changed");
Console.WriteLine("PASS: supervising process remains Medium.");
foreach (var invalid in new[] { "relative.exe", Path.Combine(root, "missing.exe") })
{
    using var rejected = Start(launcher, "--temp-parent", root, "--", invalid);
    Require(rejected.WaitForExit(5000) && rejected.ExitCode == 125, "Invalid executable accepted");
}
Console.WriteLine("PASS: invalid executable paths fail closed.");
Console.WriteLine($"Evidence retained at {root}");
return 0;
