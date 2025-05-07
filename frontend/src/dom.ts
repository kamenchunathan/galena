type AttributePair = [string, string];

interface TextNode {
  Text: string;
}

interface InputNode {
  Input: {
    attributes: AttributePair[];
  };
}

interface DivNode {
  Div: {
    attributes: AttributePair[];
    children: ViewNode[];
  };
}

interface ButtonNode {
  Button: {
    attributes: AttributePair[];
    children: ViewNode[];
  };
}

type ViewNode = DivNode | TextNode | InputNode | ButtonNode;

/**
 * Main renderer function that takes the JSON output from the view function
 * and creates a DOM tree
 */
export function renderViewToDOM(
  rootElement: HTMLElement,
  viewJson: ViewNode,
  callback: (callbackId: number, value: string) => void,
): void {
  rootElement.innerHTML = "";

  const domNode = createDOMNode(viewJson, callback);
  if (domNode) {
    rootElement.appendChild(domNode);
  }
}

/**
 * Creates a DOM node from a ViewNode
 */
function createDOMNode(
  node: ViewNode,
  callback: (callbackId: number, value: string) => void,
): Node | null {
  // Handle Text nodes
  if ("Text" in node) {
    return document.createTextNode(node.Text);
  }

  // Handle Input nodes
  if ("Input" in node) {
    const inputElement = document.createElement("input");
    applyAttributes(inputElement, node.Input.attributes);
    setupEventHandlers(inputElement, node.Input.attributes, callback);
    return inputElement;
  }

  // Handle Div nodes
  if ("Div" in node) {
    const divElement = document.createElement("div");
    applyAttributes(divElement, node.Div.attributes);
    setupEventHandlers(divElement, node.Div.attributes, callback);

    // Process children recursively
    if (node.Div.children && Array.isArray(node.Div.children)) {
      node.Div.children.forEach((childNode) => {
        const childDomNode = createDOMNode(childNode, callback);
        if (childDomNode) {
          divElement.appendChild(childDomNode);
        }
      });
    }

    return divElement;
  }

  if ("Button" in node) {
    const buttonElement = document.createElement("button");
    applyAttributes(buttonElement, node.Button.attributes);
    setupEventHandlers(buttonElement, node.Button.attributes, callback);

    // Process children recursively
    if (node.Button.children && Array.isArray(node.Button.children)) {
      node.Button.children.forEach((childNode) => {
        const childDomNode = createDOMNode(childNode, callback);
        if (childDomNode) {
          buttonElement.appendChild(childDomNode);
        }
      });
    }

    return buttonElement;
  }

  console.error("Unknown node type:", node);
  return null;
}

/**
 * Applies attributes to a DOM element
 */
function applyAttributes(
  element: HTMLElement,
  attributes: AttributePair[],
): void {
  attributes.forEach(([key, value]) => {
    if (key === "class") {
      element.className = value;
    } else if (key === "val" && element instanceof HTMLInputElement) {
      element.value = value;
    } else if (!key.startsWith("on")) {
      // Skip event handlers
      element.setAttribute(key, value);
    }
  });
}

/**
 * Sets up event handlers for an element
 */
function setupEventHandlers(
  element: HTMLElement,
  attributes: AttributePair[],
  callback: (callbackId: number, value: string) => void,
): void {
  attributes.forEach(([key, value]) => {
    if (key.startsWith("on")) {
      const eventType = key.substring(2); // Remove the "on" prefix
      const callbackId = parseInt(value, 10);

      element.addEventListener(eventType, (event) => {
        let eventValue = "";

        // Extract appropriate value based on element type and event
        if (element instanceof HTMLInputElement) {
          eventValue = element.value;
        } else if (event instanceof CustomEvent && event.detail) {
          eventValue = JSON.stringify(event.detail);
        }

        // Call WASM handler function with the callback ID
        callback(callbackId, eventValue);
      });
    }
  });
}
