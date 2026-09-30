using System;
using System.IO;
using System.Text;
using System.Diagnostics;
using System.Threading;
class Fixture {
    static int Main(string[] args) {
        string output = Environment.GetEnvironmentVariable("TEST_OUTPUT");
        if (args.Length == 1 && (args[0] == "--child" || args[0] == "--child-gated")) {
            File.WriteAllText(output + ".child", Process.GetCurrentProcess().Id.ToString());
            Console.WriteLine("child holds stdout");
            if (args[0] == "--child-gated") {
                while (!File.Exists(output + ".late-go")) Thread.Sleep(10);
            } else {
                Thread.Sleep(1000);
            }
            File.WriteAllText(output + ".late", "unexpected delayed child effect");
            Thread.Sleep(29000);
            return 0;
        }
        string mode = Environment.GetEnvironmentVariable("TEST_MODE");
        if (mode == "echo") {
            using (var w = new StreamWriter(output, false, new UTF8Encoding(false))) {
                foreach (string arg in args) w.WriteLine(Convert.ToBase64String(Encoding.UTF8.GetBytes(arg)));
                w.WriteLine(Convert.ToBase64String(Encoding.UTF8.GetBytes(Console.In.ReadToEnd())));
            }
            Console.WriteLine("fixture complete");
            return 0;
        }
        File.WriteAllText(output, Process.GetCurrentProcess().Id.ToString());
        if (mode == "child-gated") {
            while (!File.Exists(output + ".go")) Thread.Sleep(10);
            var gatedChild = new ProcessStartInfo(Process.GetCurrentProcess().MainModule.FileName, "--child-gated");
            gatedChild.UseShellExecute = false;
            gatedChild.CreateNoWindow = true;
            Process.Start(gatedChild);
            while (!File.Exists(output + ".child")) Thread.Sleep(10);
            return 0;
        }
        if (mode == "child") {
            var p = new ProcessStartInfo(Process.GetCurrentProcess().MainModule.FileName, "--child");
            p.UseShellExecute = false;
            p.CreateNoWindow = true;
            Process.Start(p);
            while (!File.Exists(output + ".child")) Thread.Sleep(10);
            return 0;
        }
        if (mode == "gated-late") {
            File.WriteAllText(output + ".ready", "ready");
            while (!File.Exists(output + ".go")) Thread.Sleep(10);
            File.WriteAllText(output + ".late", "unexpected gated payload effect");
        } else if (mode == "delayed") {
            Thread.Sleep(1000);
            File.WriteAllText(output + ".late", "unexpected delayed payload effect");
        }
        Thread.Sleep(30000);
        return 0;
    }
}
