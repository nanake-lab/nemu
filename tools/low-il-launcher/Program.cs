using System.ComponentModel;
using Nemu.LowIl;

try
{
    if (!OperatingSystem.IsWindows()) throw new PlatformNotSupportedException("Windows required.");
    NativeSecurity.RequireMedium();
    var separator = Array.IndexOf(args, "--");
    if (separator != 0 && separator != 2 || separator + 1 >= args.Length)
        throw new ArgumentException("Usage: LowIlLauncher [--temp-parent ABSOLUTE_DIR] -- ABSOLUTE_EXE [ARGS]");
    if (separator == 2 && args[0] != "--temp-parent") throw new ArgumentException("Unknown option.");
    var parent = Arguments.ExistingLocalPath(separator == 2 ? args[1] : NativeSecurity.LocalLow(), true);
    var application = Arguments.ExistingLocalPath(args[separator + 1], false);
    if (!string.Equals(Path.GetExtension(application), ".exe", StringComparison.OrdinalIgnoreCase))
        throw new ArgumentException("An executable is required; shell scripts are not accepted.");
    var workerArguments = args[(separator + 2)..];
    // Validate arguments before creating anything or lowering this disposable process.
    foreach (var argument in workerArguments) Arguments.Quote(argument);
    var systemRoot = Environment.GetFolderPath(Environment.SpecialFolder.Windows);
    using var job = new NativeProcess();
    var directory = NativeSecurity.CreateWorkerDirectory(parent);
    Console.WriteLine($"worker_temp={directory}");
    NativeSecurity.LowerSelf();
    return job.Run(application, workerArguments, directory, systemRoot);
}
catch (Exception error) when (error is ArgumentException or InvalidOperationException
    or Win32Exception or IOException or UnauthorizedAccessException or PlatformNotSupportedException)
{
    // Do not print worker arguments, environment values or arbitrary exception messages.
    Console.Error.WriteLine(error is Win32Exception native
        ? $"Low IL launcher failed: Win32 {native.NativeErrorCode}."
        : $"Low IL launcher failed: {error.GetType().Name}.");
    return 125;
}
