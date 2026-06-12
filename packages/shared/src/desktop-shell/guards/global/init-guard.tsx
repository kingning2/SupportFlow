"use client";

import { useEffect } from "react";

import { useAppDispatch, useAppSelector } from "../../store/hooks";
import { changeInitializedAction } from "../../store/modules/app";

export default function InitGuard({ children }: { children: React.ReactNode }) {
  const dispatch = useAppDispatch();
  const initialized = useAppSelector((state) => state.app.initialized);

  useEffect(() => {
    dispatch(changeInitializedAction(true));
  }, [dispatch]);

  if (!initialized) return null;

  return <>{children}</>;
}
