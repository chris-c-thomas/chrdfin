export interface NavigationItem {
  readonly id: string;
  readonly label: string;
  readonly path: string;
  readonly iconName: string;
  readonly featureFlag: string;
}

export interface NavigationSection {
  readonly id: string;
  readonly label: string;
  readonly items: readonly NavigationItem[];
}

export interface DomainManifest {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly icon: string;
  readonly basePath: string;
  readonly navigationItems: readonly NavigationItem[];
  readonly featureFlag: string;
  readonly dependencies: readonly string[];
}
