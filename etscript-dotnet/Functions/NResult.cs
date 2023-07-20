namespace Functions;

using System.Runtime.InteropServices;

internal enum Status
{
    Ok,
    Error
}

[StructLayout(LayoutKind.Sequential)]
public readonly struct NResult
{
    public NResult(string value, long data, int status)
    {
        Value = value;
        Data = data; // general-purpose field for numeric data (e.g., unix time)
        Status = status;
    }

    public string Value { get; }
    public long Data { get; }
    public int Status { get; }
}

public static class Ffi
{
    [UnmanagedCallersOnly(EntryPoint = "free_n_result")]
    public static void FreeNResult(nint ptr)
    {
        Marshal.FreeHGlobal(ptr);
    }
}
