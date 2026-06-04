"use client";

import { Button } from "@supportflow/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@supportflow/ui/card";

export function DemoPanel() {
  return (
    <div className="flex min-h-0 flex-1 items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Demo Modal</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-muted-foreground text-sm">
            This panel is mounted through the modal window registry.
          </p>
          <Button type="button">OK</Button>
        </CardContent>
      </Card>
    </div>
  );
}
