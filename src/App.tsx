import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Layout } from "./components/Layout";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./components/ui/select";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { Download, Loader2, Info } from "lucide-react";
import { toast } from "sonner";

interface QualityOption {
  id: string;
  label: string;
  format_type: string;
}

function App() {
  const [url, setUrl] = useState("");
  const [format, setFormat] = useState("video+audio");
  const [quality, setQuality] = useState("best");
  const [isDownloading, setIsDownloading] = useState(false);
  const [isFetchingInfo, setIsFetchingInfo] = useState(false);
  const [availableQualities, setAvailableQualities] = useState<QualityOption[]>([]);

  async function handleFetchVideoInfo() {
    if (!url) {
      toast.error("Please enter a YouTube URL");
      return;
    }

    setIsFetchingInfo(true);
    try {
      const qualities = await invoke<QualityOption[]>("get_video_info", { url });
      setAvailableQualities(qualities);
      toast.success("Video info fetched successfully!");
    } catch (error) {
      console.error(error);
      toast.error(`Failed to fetch video info: ${error}`);
    } finally {
      setIsFetchingInfo(false);
    }
  }

  async function handleDownload() {
    if (!url) {
      toast.error("Please enter a YouTube URL");
      return;
    }

    setIsDownloading(true);
    try {
      await invoke("download_media", { url, format, quality });
      toast.success("Download started successfully!");
    } catch (error) {
      console.error(error);
      toast.error(`Failed to start download: ${error}`);
    } finally {
      setIsDownloading(false);
    }
  }

  return (
    <Layout>
      <div className="container mx-auto p-6 max-w-5xl h-full flex flex-col justify-center">
        <Card className="border-none shadow-none bg-transparent">
          <CardHeader className="text-center space-y-2">
            <CardTitle className="text-3xl font-bold tracking-tight">
              Frieren Downloader
            </CardTitle>
            <CardDescription className="text-lg">
              Download your favorite content from YouTube with ease.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6 mt-4">
            <div className="space-y-2">
              <Label htmlFor="url">YouTube URL</Label>
              <div className="flex gap-2">
                <Input
                  id="url"
                  placeholder="https://www.youtube.com/watch?v=..."
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  className="h-11"
                />
                <Button
                  onClick={handleFetchVideoInfo}
                  disabled={isFetchingInfo}
                  variant="outline"
                  className="h-11 px-4"
                >
                  {isFetchingInfo ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Info className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Format</Label>
                <Select value={format} onValueChange={setFormat}>
                  <SelectTrigger className="h-11">
                    <SelectValue placeholder="Select format" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="video+audio">Video + Audio</SelectItem>
                    <SelectItem value="audio">Audio Only</SelectItem>
                    <SelectItem value="video">Video Only</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>Quality</Label>
                <Select value={quality} onValueChange={setQuality}>
                  <SelectTrigger className="h-11">
                    <SelectValue placeholder="Select quality" />
                  </SelectTrigger>
                  <SelectContent>
                    {availableQualities.length > 0 ? (
                      availableQualities.map((q) => (
                        <SelectItem key={q.id} value={q.id}>
                          {q.label}
                        </SelectItem>
                      ))
                    ) : (
                      <>
                        <SelectItem value="best">Best Available</SelectItem>
                        <SelectItem value="worst">Lowest Available</SelectItem>
                      </>
                    )}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <Button
              className="w-full h-11 text-lg font-medium transition-all hover:scale-[1.02] active:scale-[0.98]"
              onClick={handleDownload}
              disabled={isDownloading}
            >
              {isDownloading ? (
                <>
                  <Loader2 className="mr-2 h-5 w-5 animate-spin" />
                  Downloading...
                </>
              ) : (
                <>
                  <Download className="mr-2 h-5 w-5" />
                  Download
                </>
              )}
            </Button>
          </CardContent>
        </Card>
      </div>
    </Layout>
  );
}

export default App;